#!/bin/bash
# Oracle gold for skills-bench (Harbor task format) — fetch THIS task's upstream
# solution/solve.sh at the pinned ref and run it; the grader then runs the upstream
# tests against the result (gold => reward 1, no-op => 0).
#
# Fetched fresh at oracle run time (never baked into the agent image); the oracle
# runs as root with the network the sandboxed agent lacks. See core/oracle. Each
# task's gold is a single solve.sh that *derives* the answer (e.g. bike-rebalance
# runs the SCIP solver) — we never hand-port or hardcode it.
set -euo pipefail
ref="${SB_REF:?SB_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the real
# task name baked into the image (SB_TASK), falling back to EVAL_TASK_ID.
task="${SB_TASK:-${EVAL_TASK_ID:?SB_TASK/EVAL_TASK_ID not set}}"
url="https://raw.githubusercontent.com/benchflow-ai/skillsbench/${ref}/tasks/${task}/solution/solve.sh"

mkdir -p /root /output
cd /root
if command -v curl >/dev/null 2>&1; then curl -fsSL "$url"
elif command -v wget >/dev/null 2>&1; then wget -qO- "$url"
else python3 -c 'import sys,urllib.request; sys.stdout.write(urllib.request.urlopen(sys.argv[1]).read().decode())' "$url"
fi | bash
