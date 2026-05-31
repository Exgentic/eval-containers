#!/usr/bin/env bash
# oc-eval-sweep-fetch.sh — fetch results for the completed Jobs in a sweep.
#
# Reads the benchmark/agent/task/model straight off the succeeded Jobs' labels
# (the cluster is the record) and delegates each download to oc-eval-fetch.sh.
#
# Usage:
#   ./oc/oc-eval-sweep-fetch.sh --sweep-id <id>
#   ./oc/oc-eval-sweep-fetch.sh --latest

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SWEEP_ID=""; LATEST=false; NAMESPACE="exgentic-ns"; PVC="eval-output-pvc"
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

if $LATEST; then
  SWEEP_ID=$(oc get jobs -n "$NAMESPACE" -l sweep-id \
    --sort-by=.metadata.creationTimestamp \
    -o jsonpath='{.items[-1:].metadata.labels.sweep-id}')
fi
[[ -n "$SWEEP_ID" ]] || { echo "error: --sweep-id or --latest is required (no labelled sweep jobs found)" >&2; exit 1; }

log "Fetching completed jobs for sweep ${SWEEP_ID}"

# One "benchmark agent task model" line per succeeded Job, read from labels.
oc get jobs -n "$NAMESPACE" -l "sweep-id=${SWEEP_ID}" \
  -o jsonpath='{range .items[?(@.status.succeeded==1)]}{.metadata.labels.benchmark} {.metadata.labels.agent} {.metadata.labels.task} {.metadata.labels.model}{"\n"}{end}' \
| while read -r bench agent task model; do
    [[ -z "${bench:-}" ]] && continue
    log "fetch ${bench}/${task}/${agent}"
    bash "$REPO_DIR/oc/oc-eval-fetch.sh" \
      --benchmark "$bench" --agent "$agent" --model "$model" --task-id "$task" \
      --namespace "$NAMESPACE" --pvc "$PVC" --repo-dir "$REPO_DIR" \
      || log "WARN: fetch failed for ${bench}/${task}/${agent}"
  done

log "done"
