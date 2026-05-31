#!/usr/bin/env bash
# oc-eval-sweep.sh — deterministic coverage sweep across benchmarks/agents on OC.
#
# Usage:
#   ./oc/oc-eval-sweep.sh --n 4 --model gpt-5.4--bifrost [--dry-run]

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
N=""
MODEL=""
EVAL_MODEL=""
NAMESPACE="exgentic-ns"
PVC="exgentic-cos-pvc"
PERSIST=false
REBUILD=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --n)           N="$2";          shift 2 ;;
    --model)       MODEL="$2";      shift 2 ;;
    --eval-model)  EVAL_MODEL="$2"; shift 2 ;;
    --namespace)   NAMESPACE="$2";  shift 2 ;;
    --pvc)         PVC="$2";        shift 2 ;;
    --persist)     PERSIST=true;    shift ;;
    --rebuild)     REBUILD=true;    shift ;;
    --dry-run)     DRY_RUN=true;    shift ;;
    --repo-dir)    REPO_DIR="$2";   shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$N"     ]] && { echo "error: --n is required" >&2; exit 1; }
[[ -z "$MODEL" ]] && { echo "error: --model is required" >&2; exit 1; }
[[ "$N" -gt 0  ]] 2>/dev/null || { echo "error: --n must be a positive integer" >&2; exit 1; }

log() { echo "[oc-eval-sweep] $*"; }

# ── Reader pod helpers ─────────────────────────────────────────────────────────
to_image_name() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }

ensure_reader_pod() {
  local state
  state=$(oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
  if [[ "$state" != "Running" ]]; then
    [[ "$state" != "NotFound" ]] && oc delete pod eval-reader -n "$NAMESPACE" --ignore-not-found &>/dev/null
    log "Starting eval-reader pod..."
    oc apply -f "$REPO_DIR/oc/eval-reader-pod.yaml" -n "$NAMESPACE" &>/dev/null
    for i in $(seq 1 30); do
      state=$(oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
      [[ "$state" == "Running" ]] && break
      sleep 2
    done
    [[ "$state" != "Running" ]] && { log "error: eval-reader pod failed to start"; exit 1; }
    log "eval-reader pod ready."
  fi
}

result_exists() {
  local bench="$1" task_id="$2" agent="$3"
  local img_model img_bench job_name subpath
  img_model="$(to_image_name "$MODEL")"
  img_bench="$(to_image_name "$bench")"
  job_name="${img_bench}-task-${task_id}"
  subpath="runs/${bench}/${agent}/${img_model}/${task_id}/${job_name}/task/result.json"
  oc exec eval-reader -n "$NAMESPACE" -- test -f "/data/${subpath}" 2>/dev/null
}

# ── TODO: discover from filesystem ────────────────────────────────────────────
BENCHMARKS=(aime arc)
AGENTS=(codex claude-code)

NUM_BENCHMARKS=${#BENCHMARKS[@]}
NUM_AGENTS=${#AGENTS[@]}

log "Benchmarks (${NUM_BENCHMARKS}): ${BENCHMARKS[*]}"
log "Agents     (${NUM_AGENTS}): ${AGENTS[*]}"
log "Generating $N experiments..."

# ── Generate assignment triples ────────────────────────────────────────────────
# Experiment i: benchmark = i % B, task_id = i / B, agent = i % A
declare -a EXPERIMENTS=()
for (( i=0; i<N; i++ )); do
  bench="${BENCHMARKS[$(( i % NUM_BENCHMARKS ))]}"
  task_id=$(( i / NUM_BENCHMARKS ))
  agent="${AGENTS[$(( i % NUM_AGENTS ))]}"
  EXPERIMENTS+=("${bench}|${task_id}|${agent}")
done

# ── Write manifest ─────────────────────────────────────────────────────────────
SWEEPS_DIR="$REPO_DIR/sweeps"
mkdir -p "$SWEEPS_DIR"
SWEEP_ID="$(date -u +%Y%m%dT%H%M%S)--n${N}--${MODEL}"
MANIFEST="$SWEEPS_DIR/${SWEEP_ID}.json"

{
  echo "{"
  echo "  \"sweep_id\": \"${SWEEP_ID}\","
  echo "  \"n\": ${N},"
  echo "  \"model\": \"${MODEL}\","
  echo "  \"eval_model\": \"${EVAL_MODEL}\","
  echo "  \"namespace\": \"${NAMESPACE}\","
  echo "  \"experiments\": ["
  for (( i=0; i<N; i++ )); do
    IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
    comma=","
    [[ $i -eq $((N-1)) ]] && comma=""
    echo "    {\"benchmark\": \"${bench}\", \"task_id\": ${task_id}, \"agent\": \"${agent}\"}${comma}"
  done
  echo "  ]"
  echo "}"
} > "$MANIFEST"

log "Manifest written: $MANIFEST"

# ── Print plan ─────────────────────────────────────────────────────────────────
printf "\n  %4s  %-20s  %5s  %s\n" "IDX" "BENCHMARK" "TASK" "AGENT"
for (( i=0; i<N; i++ )); do
  IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
  printf "  %4d  %-20s  %5s  %s\n" "$i" "$bench" "$task_id" "$agent"
done
echo ""

if $DRY_RUN; then
  log "--dry-run: skipping job submission."
  exit 0
fi

# ── Ensure reader pod is available for result checks ─────────────────────────
ensure_reader_pod

# ── Launch all jobs in parallel (fire and forget) ─────────────────────────────
log "=== Launching $N jobs in parallel ==="

PASSTHROUGH_FLAGS=(--namespace "$NAMESPACE" --repo-dir "$REPO_DIR" --sweep-id "$SWEEP_ID")
[[ -n "$EVAL_MODEL" ]] && PASSTHROUGH_FLAGS+=(--eval-model "$EVAL_MODEL")
$PERSIST               && PASSTHROUGH_FLAGS+=(--persist --pvc "$PVC")
$REBUILD               && PASSTHROUGH_FLAGS+=(--rebuild)

OC_RUN="$REPO_DIR/oc/oc-eval-run.sh"
SWEEP_LOG_DIR="$SWEEPS_DIR/${SWEEP_ID}"
mkdir -p "$SWEEP_LOG_DIR"

for (( i=0; i<N; i++ )); do
  IFS='|' read -r bench task_id agent <<< "${EXPERIMENTS[$i]}"
  label="${bench}-${task_id}-${agent}"
  logfile="$SWEEP_LOG_DIR/${label}.log"

  CMD="bash $OC_RUN --benchmark $bench --task-id $task_id --agent $agent --model $MODEL --fire-and-forget ${PASSTHROUGH_FLAGS[*]}"

  if result_exists "$bench" "$task_id" "$agent"; then
    msg="Skipping [$i]: $label (result already exists)"
    log "$msg"
    echo "# $msg" > "$logfile"
    continue
  fi

  log "Launching [$i]: $label  →  $logfile"
  { echo "# $CMD"; bash "$OC_RUN" \
      --benchmark "$bench" \
      --task-id   "$task_id" \
      --agent     "$agent" \
      --model     "$MODEL" \
      --fire-and-forget \
      "${PASSTHROUGH_FLAGS[@]}"; } &>"$logfile" &
done

echo ""
log "=== All $N jobs launched. Sweep exiting. ==="
log "Check status : ./oc/oc-eval-sweep-status.sh --sweep-id ${SWEEP_ID}"
log "Clean up     : oc delete jobs -n ${NAMESPACE} -l sweep-id=${SWEEP_ID}"
