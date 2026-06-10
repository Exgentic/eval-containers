#!/bin/bash
# Oracle gold for terminal-bench (Harbor 2.1) — fetch THIS task's upstream
# solution/solve.sh at the pinned ref and run it in /app; the grader then runs the
# upstream tests against the result (gold => reward 1, no-op => 0).
#
# Fetched fresh at oracle run time (never baked into the agent image); the oracle
# runs as root with the network the sandboxed agent lacks. See core/oracle. Harbor
# 2.1 ships every task's gold as a single solve.sh — no per-task format branching.
set -euo pipefail
ref="${TBENCH_REF:?TBENCH_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the real
# task name baked into the image (TBENCH_TASK), falling back to EVAL_TASK_ID.
task="${TBENCH_TASK:-${EVAL_TASK_ID:?TBENCH_TASK/EVAL_TASK_ID not set}}"
url="https://raw.githubusercontent.com/harbor-framework/terminal-bench-2-1/${ref}/tasks/${task}/solution/solve.sh"

cd /app
if command -v curl >/dev/null 2>&1; then curl -fsSL "$url"
elif command -v wget >/dev/null 2>&1; then wget -qO- "$url"
else python3 -c 'import sys,urllib.request; sys.stdout.write(urllib.request.urlopen(sys.argv[1]).read().decode())' "$url"
fi | bash
