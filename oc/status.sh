#!/usr/bin/env bash
# status.sh — eval Job *run* progress off labels (one `oc get jobs`). For eval
# *results* (pass/reward/cost/traces): fetch.sh + `eval-containers report output/`.
#
#   ./oc/status.sh --sweep-id <id>   # or --benchmark aime, or nothing for all
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
[[ -z "$SELECTOR" ]] && SELECTOR="benchmark"   # any chart-made Job carries it

oc get jobs -n "$NAMESPACE" -l "$SELECTOR" \
  -o custom-columns='NAME:.metadata.name,BENCH:.metadata.labels.benchmark,AGENT:.metadata.labels.agent,MODEL:.metadata.labels.model,COMPLETIONS:.status.succeeded,TOTAL:.spec.completions,FAILED:.status.failed,ACTIVE:.status.active,SUSPENDED:.spec.suspend'
