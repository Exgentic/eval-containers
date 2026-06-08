#!/usr/bin/env bash
# status.sh — eval status: k8s Job progress + eval-level results off the PVC.
#
#   ./oc/status.sh --sweep-id <id>          # one sweep
#   ./oc/status.sh --benchmark aime         # everything for a benchmark
#   ./oc/status.sh                          # all eval Jobs in the namespace
#   ./oc/status.sh --sweep-id <id> --no-results   # Job columns only (no PVC reads)
#
# Job progress comes from labels (Indexed `succeeded/completions`). Unless
# --no-results, each Job's outputs are also read off the PVC via the eval-reader
# pod to show PASSED/TOTAL and how many runs are missing OTel LLM spans — the
# eval-health view the k8s Job status alone can't give.
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

NAMESPACE="$NS_DEFAULT" SELECTOR="" RESULTS=true
while [[ $# -gt 0 ]]; do case "$1" in
  --sweep-id) SELECTOR="sweep-id=$2"; shift 2;;
  --benchmark) SELECTOR="benchmark=$2"; shift 2;;
  --agent) SELECTOR="agent=$2"; shift 2;;
  --namespace) NAMESPACE="$2"; shift 2;;
  --no-results) RESULTS=false; shift;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$SELECTOR" ]] && SELECTOR="benchmark"   # any chart-made Job carries it

# eval-reader must be up for the PVC reads; degrade gracefully if it isn't.
if $RESULTS && ! oc get pod eval-reader -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null | grep -q Running; then
  echo "[status] note: eval-reader pod not Running — showing Job status only (run fetch.sh once, or --no-results)"; RESULTS=false
fi

# One result summary for a Job's output tree: "<passed>/<total> <noTraces>".
# Walks every */task/result.json under the run dir (works for dataset + single).
summarize() {  # $1=base dir under /data
  oc exec eval-reader -n "$NAMESPACE" -- sh -c '
    base="$1"; [ -d "$base" ] || { echo "-/- -"; exit; }
    total=0 passed=0 notr=0
    for r in $(find "$base" -path "*/task/result.json" 2>/dev/null); do
      total=$((total+1))
      grep -q "\"passed\":true" "$r" && passed=$((passed+1))
      d=$(dirname "$(dirname "$r")")
      cat "$d"/traces.jsonl "$d"/traces.json 2>/dev/null | grep -q "gen_ai" || notr=$((notr+1))
    done
    echo "$passed/$total $notr"
  ' _ "$1" 2>/dev/null || echo "-/- -"
}

printf '%-26s %-12s %-8s %-9s %-11s %-9s\n' NAME BENCH/AGENT JOBS SUSPENDED PASSED NO_TRACES
oc get jobs -n "$NAMESPACE" -l "$SELECTOR" \
  -o jsonpath='{range .items[*]}{.metadata.name}|{.metadata.labels.benchmark}|{.metadata.labels.agent}|{.metadata.labels.model}|{.metadata.labels.task}|{.status.succeeded}|{.spec.completions}|{.spec.suspend}{"\n"}{end}' \
| while IFS='|' read -r name b a m task succ comp susp; do
    [[ -z "$name" ]] && continue
    prefix="runs"; [[ "$name" == *-test ]] && prefix="runs-test"
    if [[ -n "$task" ]]; then base="/data/$prefix/$b/$a/$m/$task/$name"; else base="/data/$prefix/$b/$a/$m"; fi
    res="—  —"; $RESULTS && res=$(summarize "$base")
    printf '%-26s %-12s %-8s %-9s %-11s %-9s\n' \
      "$name" "$b/$a" "${succ:-0}/${comp:-?}" "${susp:-false}" "${res% *}" "${res##* }"
  done
