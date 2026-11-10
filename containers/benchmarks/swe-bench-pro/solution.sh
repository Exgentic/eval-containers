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

mkdir -p /solution
curl -fsSL --retry 3 --retry-delay 2 "${base}/gold_patch.diff" -o /solution/gold_patch.diff
curl -fsSL --retry 3 --retry-delay 2 "${base}/solve.sh" -o /solution/solve.sh
bash /solution/solve.sh
