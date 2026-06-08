#!/usr/bin/env bash
# status.sh — eval Job status, straight off labels. One `oc get jobs`.
#
#   ./oc/status.sh --sweep-id <id>          # one sweep
#   ./oc/status.sh --benchmark aime         # everything for a benchmark
#   ./oc/status.sh                          # all eval Jobs in the namespace
#
# Indexed Jobs report COMPLETIONS as <succeeded>/<datasetSize>, so dataset
# progress needs no manifest, reader pod, or log scraping — it's a column.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

NAMESPACE="$NS_DEFAULT" SELECTOR=""
while [[ $# -gt 0 ]]; do case "$1" in
  --sweep-id) SELECTOR="sweep-id=$2"; shift 2;;
  --benchmark) SELECTOR="benchmark=$2"; shift 2;;
  --agent) SELECTOR="agent=$2"; shift 2;;
  --namespace) NAMESPACE="$2"; shift 2;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
# Default selector: any Job the chart made (it always sets a benchmark label).
[[ -z "$SELECTOR" ]] && SELECTOR="benchmark"

oc get jobs -n "$NAMESPACE" -l "$SELECTOR" \
  -o custom-columns='NAME:.metadata.name,BENCH:.metadata.labels.benchmark,AGENT:.metadata.labels.agent,MODEL:.metadata.labels.model,COMPLETIONS:.status.succeeded,TOTAL:.spec.completions,FAILED:.status.failed,ACTIVE:.status.active,SUSPENDED:.spec.suspend'
