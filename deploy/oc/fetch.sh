#!/usr/bin/env bash
# fetch.sh — `oc cp` eval output off the PVC (paths read from Job labels).
#
#   ./oc/fetch.sh --benchmark aime --agent codex --model azure/gpt-5-mini   # whole dataset
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

copy() {  # $1=benchmark $2=agent $3=model handle (or its slug)
  local m; m="$(model_slug "$3")" ; local sub="runs/$1/$2/$m" dest="$DEST_ROOT/$1/$2/$m"
  mkdir -p "$dest"; log "oc cp $sub → $dest"
  oc cp "$NAMESPACE/eval-reader:/data/${sub}/." "$dest/" 2>/dev/null || log "  (nothing at $sub yet)"
}

copy_sub() {  # $1 = a Job's output subPath, mirrored verbatim under DEST_ROOT
  local sub="$1" dest="$DEST_ROOT/${1#runs/}"
  mkdir -p "$dest"; log "oc cp $sub → $dest"
  oc cp "$NAMESPACE/eval-reader:/data/${sub}/." "$dest/" 2>/dev/null || log "  (nothing at $sub yet)"
}

if [[ -n "$SWEEP_ID" ]]; then
  # Ask each Job where it actually wrote, rather than rebuilding the path from
  # labels: the `model` label is the handle's last segment (label values forbid
  # `/` and cap at 63 chars) while the path carries the whole slugged handle, so
  # a label-built path misses every run whose handle had a provider prefix. The
  # Job's own `output` subPath is the one source that cannot disagree.
  oc get jobs -n "$NAMESPACE" -l "sweep-id=$SWEEP_ID,benchmark" \
    -o jsonpath='{range .items[*]}{.spec.template.spec.containers[0].volumeMounts[?(@.name=="output")].subPath}{"\n"}{end}' \
    | while read -r sub; do [[ -n "$sub" ]] && copy_sub "$sub"; done
else
  [[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
    echo "error: --sweep-id, or --benchmark/--agent/--model, required" >&2; exit 1; }
  copy "$BENCHMARK" "$AGENT" "$MODEL"
fi
log "done → $DEST_ROOT"
