#!/bin/bash
# Oracle gold for SWE-bench Pro V2 — fetch THIS task's upstream reference
# solution at the pinned ref and run upstream's solve.sh, which applies it in
# /app (or /testbed). The grader then runs the hidden tests (gold => 1, no-op => 0).
#
# Fetched fresh at oracle run time, never baked (rules 9, 20a). The oracle runs
# as root with the network the sandboxed agent lacks. See core/oracle.
set -euo pipefail

ref="${SBP_REF:?SBP_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the
# instance id baked into the image (SBP_TASK), falling back to EVAL_TASK_ID (rule 24i).
task="${SBP_TASK:-${EVAL_TASK_ID:?SBP_TASK/EVAL_TASK_ID not set}}"
base="https://raw.githubusercontent.com/scaleapi/SWE-bench_Pro-os/${ref}/v2/tasks/${task}/solution"

# Not every V2 base ships curl (the qutebrowser images don't), so fall back the
# way deepswe's solution.sh does; python3 is in every base (test.sh needs it).
fetch() {
  if command -v curl >/dev/null 2>&1; then curl -fsSL --retry 3 --retry-delay 2 "$1"
  else python3 -c 'import sys,urllib.request; sys.stdout.buffer.write(urllib.request.urlopen(sys.argv[1]).read())' "$1"
  fi
}

mkdir -p /solution
fetch "${base}/gold_patch.diff" > /solution/gold_patch.diff
fetch "${base}/solve.sh"        > /solution/solve.sh
bash /solution/solve.sh
