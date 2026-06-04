#!/usr/bin/env bash
# oc-eval-sweep.sh — deterministic coverage sweep across benchmarks/agents on OC.
#
# Usage:
#   ./oc/oc-eval-sweep.sh --tasks-per-benchmark 5 --n 4 --model gpt-5.4--bifrost [--dry-run]
#
# Responsibilities (sweep only):
#   - Generate deterministic experiment list
#   - Pre-flight checks: OC login, reader pod
#   - Create a log file per experiment
#   - Fire off oc-eval-run.sh in background for each experiment
#
# Per-experiment logic (result check, image build, job submit) is all in oc-eval-run.sh.

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TASKS_PER_BENCHMARK=""
N=""
MODEL=""
EVAL_MODEL=""
NAMESPACE="exgentic-ns"
PVC="eval-output-pvc"
PERSIST=false
REBUILD=false
RERUN=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tasks-per-benchmark) TASKS_PER_BENCHMARK="$2"; shift 2 ;;
    --n)           N="$2";          shift 2 ;;
    --model)       MODEL="$2";      shift 2 ;;
    --eval-model)  EVAL_MODEL="$2"; shift 2 ;;
    --namespace)   NAMESPACE="$2";  shift 2 ;;
    --pvc)         PVC="$2";        shift 2 ;;
    --persist)     PERSIST=true;    shift ;;
    --rebuild)     REBUILD=true;    shift ;;
    --rerun)       RERUN=true;      shift ;;
    --dry-run)     DRY_RUN=true;    shift ;;
    --repo-dir)    REPO_DIR="$2";   shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$TASKS_PER_BENCHMARK" ]] && { echo "error: --tasks-per-benchmark is required" >&2; exit 1; }
[[ -z "$MODEL"               ]] && { echo "error: --model is required" >&2; exit 1; }
[[ "$TASKS_PER_BENCHMARK" -gt 0 ]] 2>/dev/null || { echo "error: --tasks-per-benchmark must be a positive integer" >&2; exit 1; }

log() { echo "[oc-eval-sweep] $*"; }

# ── Pre-flight: OC login check ────────────────────────────────────────────────
log "Checking OC login..."
if ! oc whoami &>/dev/null; then
  log "ERROR: not logged in to OpenShift (oc whoami failed). Run: oc login ..."
  exit 1
fi
log "OC login OK ($(oc whoami))"

# ── Pre-flight: reader pod ─────────────────────────────────────────────────────
log "Checking eval-reader pod..."
READER_STATE=$(oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
if [[ "$READER_STATE" != "Running" ]]; then
  [[ "$READER_STATE" != "NotFound" ]] && oc delete pod eval-reader -n "$NAMESPACE" --ignore-not-found &>/dev/null
  log "eval-reader not running (state: $READER_STATE) — starting..."
  oc apply -f "$REPO_DIR/oc/eval-reader-pod.yaml" -n "$NAMESPACE" &>/dev/null
  for i in $(seq 1 30); do
    READER_STATE=$(oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
    [[ "$READER_STATE" == "Running" ]] && break
    sleep 2
  done
  if [[ "$READER_STATE" != "Running" ]]; then
    log "ERROR: eval-reader pod failed to start (state: $READER_STATE)"
    exit 1
  fi
  log "eval-reader pod started OK"
else
  log "eval-reader pod OK (Running)"
fi

# ── Load benchmark and agent lists ────────────────────────────────────────────
BENCHMARKS_FILE="$REPO_DIR/oc/benchmarks.txt"
AGENTS_FILE="$REPO_DIR/oc/agents.txt"

if [[ ! -f "$BENCHMARKS_FILE" ]]; then
  log "benchmarks.txt not found — running discovery..."
  bash "$REPO_DIR/oc/discover-benchmarks.sh" > "$BENCHMARKS_FILE"
  log "Created $BENCHMARKS_FILE. To refresh: bash oc/discover-benchmarks.sh > oc/benchmarks.txt"
else
  log "Benchmarks loaded from $BENCHMARKS_FILE (to refresh: bash oc/discover-benchmarks.sh > oc/benchmarks.txt)"
fi

if [[ ! -f "$AGENTS_FILE" ]]; then
  log "agents.txt not found — running discovery..."
  bash "$REPO_DIR/oc/discover-agents.sh" > "$AGENTS_FILE"
  log "Created $AGENTS_FILE. To refresh: bash oc/discover-agents.sh > oc/agents.txt"
else
  log "Agents loaded from $AGENTS_FILE (to refresh: bash oc/discover-agents.sh > oc/agents.txt)"
fi

mapfile -t BENCHMARKS < <(grep -v '^\s*#' "$BENCHMARKS_FILE" | grep -v '^\s*$')
mapfile -t AGENTS    < <(grep -v '^\s*#' "$AGENTS_FILE"     | grep -v '^\s*$')

NUM_BENCHMARKS=${#BENCHMARKS[@]}
NUM_AGENTS=${#AGENTS[@]}
TOTAL=$(( NUM_BENCHMARKS * TASKS_PER_BENCHMARK ))

# N defaults to total plan size; clamp if user specified more than total
if [[ -z "$N" ]]; then
  N=$TOTAL
else
  [[ "$N" -gt 0 ]] 2>/dev/null || { echo "error: --n must be a positive integer" >&2; exit 1; }
  [[ "$N" -gt "$TOTAL" ]] && { echo "error: --n ($N) exceeds total plan size ($TOTAL)" >&2; exit 1; }
fi

log "Benchmarks (${NUM_BENCHMARKS}): ${BENCHMARKS[*]}"
log "Agents     (${NUM_AGENTS}): ${AGENTS[*]}"
log "Tasks per benchmark: $TASKS_PER_BENCHMARK  |  Total plan: $TOTAL  |  Submitting: $N"

# ── Generate assignment triples ────────────────────────────────────────────────
# Plan: for each benchmark b (index), for each task t in 0..TPB-1:
#   agent = (b * TASKS_PER_BENCHMARK + t) % NUM_AGENTS
# This maximises agent spread per benchmark and keeps ordering deterministic.
declare -a EXPERIMENTS=()
for (( b=0; b<NUM_BENCHMARKS; b++ )); do
  for (( t=0; t<TASKS_PER_BENCHMARK; t++ )); do
    bench="${BENCHMARKS[$b]}"
    task_id="$t"
    agent="${AGENTS[$(( (b * TASKS_PER_BENCHMARK + t) % NUM_AGENTS ))]}"
    EXPERIMENTS+=("${bench}|${task_id}|${agent}")
  done
done

# ── Write manifest ─────────────────────────────────────────────────────────────
SWEEPS_DIR="$REPO_DIR/sweeps"
mkdir -p "$SWEEPS_DIR"
SWEEP_ID="$(date -u +%Y%m%dT%H%M%S)--tpb${TASKS_PER_BENCHMARK}--${MODEL}"
MANIFEST="$SWEEPS_DIR/${SWEEP_ID}.json"

{
  echo "{"
  echo "  \"sweep_id\": \"${SWEEP_ID}\","
  echo "  \"tasks_per_benchmark\": ${TASKS_PER_BENCHMARK},"
  echo "  \"n\": ${N},"
  echo "  \"total_plan\": ${TOTAL},"
  echo "  \"model\": \"${MODEL}\","
  echo "  \"eval_model\": \"${EVAL_MODEL}\","
  echo "  \"namespace\": \"${NAMESPACE}\","
  echo "  \"experiments\": ["
  for (( i=0; i<TOTAL; i++ )); do
    IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
    comma=","
    [[ $i -eq $((TOTAL-1)) ]] && comma=""
    echo "    {\"benchmark\": \"${bench}\", \"task_id\": ${task_id}, \"agent\": \"${agent}\"}${comma}"
  done
  echo "  ]"
  echo "}"
} > "$MANIFEST"

log "Manifest written: $MANIFEST"

# ── Print plan ─────────────────────────────────────────────────────────────────
printf "\n  %4s  %-20s  %5s  %s\n" "IDX" "BENCHMARK" "TASK" "AGENT"
for (( i=0; i<TOTAL; i++ )); do
  IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
  marker=""; [[ $i -ge $N ]] && marker=" (plan-only)"
  printf "  %4d  %-20s  %5s  %s%s\n" "$i" "$bench" "$task_id" "$agent" "$marker"
done
echo ""

if $DRY_RUN; then
  log "--dry-run: skipping job submission."
  exit 0
fi

# ── Launch all jobs in parallel (fire and forget) ─────────────────────────────
log "=== Launching $N jobs in parallel ==="

PASSTHROUGH_FLAGS=(--namespace "$NAMESPACE" --repo-dir "$REPO_DIR" --sweep-id "$SWEEP_ID")
[[ -n "$EVAL_MODEL" ]] && PASSTHROUGH_FLAGS+=(--eval-model "$EVAL_MODEL")
$PERSIST               && PASSTHROUGH_FLAGS+=(--persist --pvc "$PVC")
$REBUILD               && PASSTHROUGH_FLAGS+=(--rebuild)
$RERUN                 && PASSTHROUGH_FLAGS+=(--rerun)

OC_RUN="$REPO_DIR/oc/oc-eval-run.sh"
SWEEP_LOG_DIR="$SWEEPS_DIR/${SWEEP_ID}"
mkdir -p "$SWEEP_LOG_DIR"

for (( i=0; i<N; i++ )); do
  IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
  label="${bench}-${task_id}-${agent}"
  logfile="$SWEEP_LOG_DIR/${label}.log"

  log "Launching [$i]: $label  →  $logfile"
  {
    echo "# [oc-eval-sweep] launched: $label at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    bash "$OC_RUN" \
      --benchmark "$bench" \
      --task-id   "$task_id" \
      --agent     "$agent" \
      --model     "$MODEL" \
      --fire-and-forget \
      "${PASSTHROUGH_FLAGS[@]}"
  } &>"$logfile" &
done

echo ""
log "=== $N / $TOTAL jobs launched. Sweep exiting. ==="
log "Check status : ./oc/oc-eval-sweep-status.sh --sweep-id ${SWEEP_ID}"
log "Clean up     : oc delete jobs -n ${NAMESPACE} -l sweep-id=${SWEEP_ID}"
