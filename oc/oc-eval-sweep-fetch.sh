#!/usr/bin/env bash
# oc-eval-sweep-fetch.sh — download results for all completed experiments in a sweep.
#
# Usage:
#   ./oc/oc-eval-sweep-fetch.sh --sweep-id 20260528T120000--n4--gpt-5.4--bifrost
#   ./oc/oc-eval-sweep-fetch.sh --latest

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SWEEP_ID=""
LATEST=false
NAMESPACE=""
PVC="eval-output-pvc"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sweep-id)  SWEEP_ID="$2";  shift 2 ;;
    --latest)    LATEST=true;    shift ;;
    --namespace) NAMESPACE="$2"; shift 2 ;;
    --pvc)       PVC="$2";       shift 2 ;;
    --repo-dir)  REPO_DIR="$2";  shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

log() { echo "[oc-eval-sweep-fetch] $*"; }

SWEEPS_DIR="$REPO_DIR/sweeps"

# ── Resolve manifest ───────────────────────────────────────────────────────────
if $LATEST; then
  MANIFEST=$(ls -t "$SWEEPS_DIR"/*.json 2>/dev/null | head -1)
  [[ -z "$MANIFEST" ]] && { echo "error: no sweep manifests found in $SWEEPS_DIR" >&2; exit 1; }
  SWEEP_ID="$(basename "$MANIFEST" .json)"
  log "Using latest sweep: $SWEEP_ID"
elif [[ -n "$SWEEP_ID" ]]; then
  MANIFEST="$SWEEPS_DIR/${SWEEP_ID}.json"
else
  echo "error: --sweep-id or --latest is required" >&2; exit 1
fi

[[ -f "$MANIFEST" ]] || { echo "error: manifest not found: $MANIFEST" >&2; exit 1; }

manifest_ns=$(grep '"namespace"' "$MANIFEST" | sed 's/.*: *"\(.*\)".*/\1/')
manifest_model=$(grep '"model"' "$MANIFEST" | sed 's/.*: *"\(.*\)".*/\1/')
NS="${NAMESPACE:-$manifest_ns}"
NS="${NS:-exgentic-ns}"
MODEL="$manifest_model"

log "Sweep:     $SWEEP_ID"
log "Namespace: $NS"
log "Model:     $MODEL"
echo ""

OC_FETCH="$REPO_DIR/oc/oc-eval-fetch.sh"

FETCHED=0
SKIPPED=0

# ── Fetch each completed experiment ───────────────────────────────────────────
while IFS= read -r line; do
  bench=$(echo "$line"  | sed 's/.*"benchmark": *"\([^"]*\)".*/\1/')
  task=$(echo "$line"   | sed 's/.*"task_id": *\([0-9]*\).*/\1/')
  agent=$(echo "$line"  | sed 's/.*"agent": *"\([^"]*\)".*/\1/')

  job_name="${bench}-${agent}-task-${task}"

  status=$(oc get job "$job_name" -n "$NS" \
    -o jsonpath='{.status.conditions[*].type}' 2>/dev/null || echo "NotFound")

  if [[ "$status" == *"Complete"* ]]; then
    log "Fetching: $bench / task=$task / $agent"
    bash "$OC_FETCH" \
      --benchmark "$bench" \
      --agent     "$agent" \
      --model     "$MODEL" \
      --task-id   "$task" \
      --namespace "$NS" \
      --pvc       "$PVC" \
      --repo-dir  "$REPO_DIR"
    (( FETCHED++ )) || true
  else
    log "Skipping ($status): $bench / task=$task / $agent"
    (( SKIPPED++ )) || true
  fi
done < <(grep '"benchmark"' "$MANIFEST")

echo ""
log "=== Done: $FETCHED fetched, $SKIPPED skipped ==="
