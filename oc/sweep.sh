#!/usr/bin/env bash
# sweep.sh — loop run.sh over a benchmark×agent grid, each cell a dataset Indexed
# Job tagged sweep-id=<id>. Flags: see the case block. Default grid: the *.txt.
#
#   ./oc/sweep.sh --dataset-size 50 --model gpt-5.4--bifrost --queue eval-queue
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"
RUN="$(dirname "${BASH_SOURCE[0]}")/run.sh"

MODEL="" DATASET="" PARALLELISM="" RETRY="" QUEUE="" BSET="" ASET=""
EVAL_MODEL="" NAMESPACE="$NS_DEFAULT" PVC="eval-output-pvc" NO_BUILD=false DRY_RUN=false
while [[ $# -gt 0 ]]; do case "$1" in
  --model) MODEL="$2"; shift 2;; --dataset-size) DATASET="$2"; shift 2;;
  --parallelism) PARALLELISM="$2"; shift 2;; --retry) RETRY="$2"; shift 2;;
  --queue) QUEUE="$2"; shift 2;; --benchmarks) BSET="$2"; shift 2;; --agents) ASET="$2"; shift 2;;
  --eval-model) EVAL_MODEL="$2"; shift 2;; --namespace) NAMESPACE="$2"; shift 2;;
  --pvc) PVC="$2"; shift 2;; --repo-dir) REPO_DIR="$2"; shift 2;;
  --no-build) NO_BUILD=true; shift;; --dry-run) DRY_RUN=true; shift;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$MODEL" ]] && { echo "error: --model is required" >&2; exit 1; }
log() { echo "[sweep] $*"; }

read_list() { grep -v '^[[:space:]]*#' "$1" | grep -v '^[[:space:]]*$'; }
if [[ -n "$BSET" ]]; then read -ra BENCHMARKS <<<"$BSET"
else BENCHMARKS=(); while IFS= read -r l; do BENCHMARKS+=("$l"); done < <(read_list "$REPO_DIR/oc/benchmarks.txt"); fi
if [[ -n "$ASET" ]]; then read -ra AGENTS <<<"$ASET"
else AGENTS=(); while IFS= read -r l; do AGENTS+=("$l"); done < <(read_list "$REPO_DIR/oc/agents.txt"); fi

SWEEP_ID="$(date -u +%Y%m%dT%H%M%S)--$(flat "$MODEL")"
log "sweep-id: $SWEEP_ID   grid: ${#BENCHMARKS[@]} benchmarks × ${#AGENTS[@]} agents${QUEUE:+   queue: $QUEUE}"

PASS=(--model "$MODEL" --namespace "$NAMESPACE" --pvc "$PVC" --repo-dir "$REPO_DIR" --sweep-id "$SWEEP_ID")
[[ -n "$DATASET"     ]] && PASS+=(--dataset-size "$DATASET")
[[ -n "$PARALLELISM" ]] && PASS+=(--parallelism "$PARALLELISM")
[[ -n "$RETRY"       ]] && PASS+=(--retry "$RETRY")
[[ -n "$QUEUE"       ]] && PASS+=(--queue "$QUEUE")
[[ -n "$EVAL_MODEL"  ]] && PASS+=(--eval-model "$EVAL_MODEL")
$NO_BUILD && PASS+=(--no-build)
$DRY_RUN  && PASS+=(--dry-run)

for b in "${BENCHMARKS[@]}"; do for a in "${AGENTS[@]}"; do
  log "→ $b × $a"
  bash "$RUN" --benchmark "$b" --agent "$a" "${PASS[@]}"
done; done

log "=== submitted ${#BENCHMARKS[@]}×${#AGENTS[@]} jobs ==="
log "status: ./oc/status.sh --sweep-id $SWEEP_ID"
log "fetch : ./oc/fetch.sh  --sweep-id $SWEEP_ID"
log "clean : oc delete jobs -n $NAMESPACE -l sweep-id=$SWEEP_ID"
