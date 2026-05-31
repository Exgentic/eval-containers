#!/usr/bin/env bash
# oc-eval-sweep.sh — launch a coverage sweep (benchmark × agent × task) on OC.
#
# Builds each distinct benchmark×agent eval image once, up front (fail fast),
# then submits one labelled, run-only k8s Job per experiment. The cluster is the
# record — track/collect with oc-eval-sweep-status.sh / -fetch.sh (by sweep-id).
#
# --benchmark / --agent pin an axis of the grid; --skip-build assumes the eval
# images are already built and pushed.
#
# Usage:
#   ./oc/oc-eval-sweep.sh --n 4 --model gpt-5.4--bifrost \
#       [--benchmark aime] [--agent codex] [--max-parallel 8] [--skip-build] [--dry-run]

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
N=""; MODEL=""; EVAL_MODEL=""; NAMESPACE="exgentic-ns"; REGISTRY=""; BUILDER=""
ONLY_BENCH=""; ONLY_AGENT=""; PVC="exgentic-cos-pvc"
PERSIST=false; REBUILD=false; SKIP_BUILD=false; DRY_RUN=false; MAX_PARALLEL=8

while [[ $# -gt 0 ]]; do
  case "$1" in
    --n)            N="$2";            shift 2 ;;
    --model)        MODEL="$2";        shift 2 ;;
    --eval-model)   EVAL_MODEL="$2";   shift 2 ;;
    --benchmark)    ONLY_BENCH="$2";   shift 2 ;;
    --agent)        ONLY_AGENT="$2";   shift 2 ;;
    --namespace)    NAMESPACE="$2";    shift 2 ;;
    --registry)     REGISTRY="$2";     shift 2 ;;
    --builder)      BUILDER="$2";      shift 2 ;;
    --pvc)          PVC="$2";          shift 2 ;;
    --persist)      PERSIST=true;      shift ;;
    --rebuild)      REBUILD=true;      shift ;;
    --skip-build)   SKIP_BUILD=true;   shift ;;
    --dry-run)      DRY_RUN=true;      shift ;;
    --max-parallel) MAX_PARALLEL="$2"; shift 2 ;;
    --repo-dir)     REPO_DIR="$2";     shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -n "$N"     ]] || { echo "error: --n is required" >&2; exit 1; }
[[ -n "$MODEL" ]] || { echo "error: --model is required" >&2; exit 1; }
[[ "$N" -gt 0 ]] 2>/dev/null            || { echo "error: --n must be a positive integer" >&2; exit 1; }
[[ "$MAX_PARALLEL" -gt 0 ]] 2>/dev/null || { echo "error: --max-parallel must be a positive integer" >&2; exit 1; }

log() { echo "[oc-eval-sweep] $*"; }
OC_RUN="$REPO_DIR/oc/oc-eval-run.sh"

# Sweep set: --benchmark/--agent pin an axis; else discover everything with a
# Dockerfile (oc-eval-run.sh deploys from the shared benchmarks/_base, so any
# buildable benchmark works). Globs sort alphabetically → deterministic.
if [[ -n "$ONLY_BENCH" ]]; then
  [[ -f "$REPO_DIR/benchmarks/$ONLY_BENCH/Dockerfile" ]] || { echo "error: benchmarks/$ONLY_BENCH has no Dockerfile" >&2; exit 1; }
  BENCHMARKS=("$ONLY_BENCH")
else
  BENCHMARKS=(); for d in "$REPO_DIR"/benchmarks/*/; do [[ -f "${d}Dockerfile" ]] && BENCHMARKS+=("$(basename "$d")"); done
fi
if [[ -n "$ONLY_AGENT" ]]; then
  [[ -f "$REPO_DIR/agents/$ONLY_AGENT/Dockerfile" ]] || { echo "error: agents/$ONLY_AGENT has no Dockerfile" >&2; exit 1; }
  AGENTS=("$ONLY_AGENT")
else
  AGENTS=(); for d in "$REPO_DIR"/agents/*/; do [[ -f "${d}Dockerfile" ]] && AGENTS+=("$(basename "$d")"); done
fi
B=${#BENCHMARKS[@]}; A=${#AGENTS[@]}
[[ $B -gt 0 ]] || { echo "error: no benchmarks found" >&2; exit 1; }
[[ $A -gt 0 ]] || { echo "error: no agents found" >&2; exit 1; }

SWEEP_ID="$(date -u +%Y%m%dT%H%M%S)--n${N}--${MODEL}"
LOG_DIR="$REPO_DIR/sweeps/${SWEEP_ID}"; mkdir -p "$LOG_DIR"

# Enumerate the full benchmark × agent × task product (bench fastest, then
# agent, then task — covers the grid, no diagonal coupling). Collect the
# experiments and the distinct (benchmark, agent) pairs to build.
log "Sweep $SWEEP_ID — $B benchmarks × $A agents, $N experiments"
printf "\n  %4s  %-20s  %5s  %s\n" IDX BENCHMARK TASK AGENT
EXPERIMENTS=(); PAIRS=()
for (( i=0; i<N; i++ )); do
  bench="${BENCHMARKS[$(( i % B ))]}"
  agent="${AGENTS[$(( (i / B) % A ))]}"
  task=$(( i / (B * A) ))
  printf "  %4d  %-20s  %5s  %s\n" "$i" "$bench" "$task" "$agent"
  EXPERIMENTS+=("${bench}|${task}|${agent}")
  PAIRS+=("${bench}|${agent}")
done
echo ""
$DRY_RUN && { log "--dry-run: nothing built or submitted."; exit 0; }

COMMON=(--model "$MODEL" --namespace "$NAMESPACE" --repo-dir "$REPO_DIR")
[[ -n "$EVAL_MODEL" ]] && COMMON+=(--eval-model "$EVAL_MODEL")
[[ -n "$REGISTRY"   ]] && COMMON+=(--registry "$REGISTRY")
[[ -n "$BUILDER"    ]] && COMMON+=(--builder "$BUILDER")

# ── Build once, up front: each distinct benchmark×agent image; fail fast ──────
if $SKIP_BUILD; then
  log "--skip-build: assuming eval images are already pushed."
else
  # ${REB[@]+...} so an empty array is not "unbound" under `set -u` on bash 3.2 (macOS).
  REB=(); $REBUILD && REB=(--rebuild)
  for pair in $(printf '%s\n' "${PAIRS[@]}" | sort -u); do
    bench="${pair%|*}"; agent="${pair#*|}"
    log "build ${bench} × ${agent}"
    bash "$OC_RUN" --benchmark "$bench" --agent "$agent" "${COMMON[@]}" ${REB[@]+"${REB[@]}"} --no-run \
      || { echo "error: build failed for ${bench} × ${agent}; aborting before submitting jobs" >&2; exit 1; }
  done
  log "All images built."
fi

# ── Run: one labelled, run-only Job per experiment, capped at --max-parallel ──
RUN=(--sweep-id "$SWEEP_ID" --fire-and-forget --no-build)
$PERSIST && RUN+=(--persist --pvc "$PVC")
log "Submitting $N jobs (max $MAX_PARALLEL parallel) ..."
for exp in "${EXPERIMENTS[@]}"; do
  IFS='|' read -r bench task agent <<< "$exp"
  # Resume: skip if this experiment's Job already succeeded (labels are the record).
  if [[ "$(oc get jobs -n "$NAMESPACE" -l "benchmark=${bench},agent=${agent},task=${task}" \
            -o jsonpath='{.items[*].status.succeeded}' 2>/dev/null)" == *1* ]]; then
    log "skip   ${bench}/${task}/${agent} (already succeeded)"; continue
  fi
  while (( $(jobs -rp | wc -l) >= MAX_PARALLEL )); do sleep 1; done
  log "launch ${bench}/${task}/${agent}"
  bash "$OC_RUN" --benchmark "$bench" --task-id "$task" --agent "$agent" \
    "${COMMON[@]}" "${RUN[@]}" &>"$LOG_DIR/${bench}-${task}-${agent}.log" &
done
wait

log "Submitted. Track by sweep-id:"
log "  status : ./oc/oc-eval-sweep-status.sh --sweep-id ${SWEEP_ID}"
log "  fetch  : ./oc/oc-eval-sweep-fetch.sh  --sweep-id ${SWEEP_ID}"
log "  logs   : oc logs -n ${NAMESPACE} -l sweep-id=${SWEEP_ID} --all-containers --prefix"
log "  clean  : oc delete jobs -n ${NAMESPACE} -l sweep-id=${SWEEP_ID}"
