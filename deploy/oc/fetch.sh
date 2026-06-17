#!/usr/bin/env bash
# fetch.sh — `oc cp` eval output off the PVC (paths read from Job labels).
#
#   ./oc/fetch.sh --benchmark aime --agent codex --model bifrost   # whole dataset
#   ./oc/fetch.sh --sweep-id <id>                                           # every Job in a sweep
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

NAMESPACE="$NS_DEFAULT" BENCHMARK="" AGENT="" MODEL="" SWEEP_ID="" DEST_ROOT=""
while [[ $# -gt 0 ]]; do case "$1" in
  --benchmark) BENCHMARK="$2"; shift 2;; --agent) AGENT="$2"; shift 2;;
  --model) MODEL="$2"; shift 2;; --sweep-id) SWEEP_ID="$2"; shift 2;;
  --namespace) NAMESPACE="$2"; shift 2;; --output-dir) DEST_ROOT="$2"; shift 2;;
  --repo-dir) REPO_DIR="$2"; shift 2;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
log() { echo "[fetch] $*"; }
DEST_ROOT="${DEST_ROOT:-$REPO_DIR/output}"

# Ensure the shared reader pod is up (idempotent).
oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null | grep -q Running \
  || { log "starting eval-reader pod…"; oc apply -f "$REPO_DIR/deploy/eval-reader-pod.yaml" -n "$NAMESPACE" >/dev/null
       oc wait --for=condition=ready pod/eval-reader -n "$NAMESPACE" --timeout=60s >/dev/null; }

copy() {  # $1=benchmark $2=agent $3=model
  local sub="runs/$1/$2/$3" dest="$DEST_ROOT/$1/$2/$3"
  mkdir -p "$dest"; log "oc cp $sub → $dest"
  oc cp "$NAMESPACE/eval-reader:/data/${sub}/." "$dest/" 2>/dev/null || log "  (nothing at $sub yet)"
}

if [[ -n "$SWEEP_ID" ]]; then
  oc get jobs -n "$NAMESPACE" -l "sweep-id=$SWEEP_ID,benchmark" \
    -o jsonpath='{range .items[*]}{.metadata.labels.benchmark} {.metadata.labels.agent} {.metadata.labels.model}{"\n"}{end}' \
    | while read -r b a m; do [[ -n "$b" ]] && copy "$b" "$a" "$m"; done
else
  [[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
    echo "error: --sweep-id, or --benchmark/--agent/--model, required" >&2; exit 1; }
  copy "$BENCHMARK" "$AGENT" "$MODEL"
fi
log "done → $DEST_ROOT"
