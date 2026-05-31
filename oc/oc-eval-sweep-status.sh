#!/usr/bin/env bash
# oc-eval-sweep-status.sh — sweep status straight from the cluster.
#
# The Jobs carry benchmark/agent/task/sweep-id labels, so status is one
# label-selected `oc get jobs` — no local manifest, no log scraping.
#
# Usage:
#   ./oc/oc-eval-sweep-status.sh --sweep-id <id>
#   ./oc/oc-eval-sweep-status.sh --latest

set -euo pipefail

SWEEP_ID=""; LATEST=false; NAMESPACE="exgentic-ns"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --sweep-id)  SWEEP_ID="$2";  shift 2 ;;
    --latest)    LATEST=true;    shift ;;
    --namespace) NAMESPACE="$2"; shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

if $LATEST; then
  SWEEP_ID=$(oc get jobs -n "$NAMESPACE" -l sweep-id \
    --sort-by=.metadata.creationTimestamp \
    -o jsonpath='{.items[-1:].metadata.labels.sweep-id}')
fi
[[ -n "$SWEEP_ID" ]] || { echo "error: --sweep-id or --latest is required (no labelled sweep jobs found)" >&2; exit 1; }

echo "[oc-eval-sweep-status] sweep ${SWEEP_ID} (namespace ${NAMESPACE})"
echo ""
oc get jobs -n "$NAMESPACE" -l "sweep-id=${SWEEP_ID}" \
  --sort-by=.metadata.creationTimestamp \
  -o custom-columns='BENCHMARK:.metadata.labels.benchmark,TASK:.metadata.labels.task,AGENT:.metadata.labels.agent,STATUS:.status.conditions[*].type,AGE:.metadata.creationTimestamp'
echo ""
echo "[oc-eval-sweep-status] STATUS blank = still running; pod-level detail: oc get pods -n ${NAMESPACE} -l sweep-id=${SWEEP_ID}"
echo "[oc-eval-sweep-status] logs: oc logs -n ${NAMESPACE} -l sweep-id=${SWEEP_ID} --all-containers --prefix --tail=20"
