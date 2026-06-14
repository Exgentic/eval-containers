#!/bin/bash
# Oracle gold for skills-bench (Harbor task format) — stage THIS task's upstream
# solution/ tree at /solution and run its solve.sh; the grader then runs the
# upstream tests against the result (gold => reward 1, no-op => 0).
#
# We stage the whole solution/ directory, not just solve.sh, because some tasks'
# solve.sh read sibling files from it (e.g. civ6 reads /solution/ground_truths/).
# Fetched fresh at oracle run time, never baked into the agent image; the oracle
# runs as root with the network the sandboxed agent lacks. See core/oracle. Each
# task's solve.sh *derives* the answer (e.g. bike-rebalance runs the SCIP solver)
# — never hand-ported or hardcoded.
set -euo pipefail
ref="${SB_REF:?SB_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the real
# task name baked into the image (SB_TASK), falling back to EVAL_TASK_ID.
task="${SB_TASK:-${EVAL_TASK_ID:?SB_TASK/EVAL_TASK_ID not set}}"

mkdir -p /root /output
rm -rf /solution && mkdir -p /solution
# GitHub tarball at the pinned commit; extract only this task's solution/ subtree
# into /solution (so /solution/solve.sh and any sibling data land there).
tmp="$(mktemp)"
url="https://github.com/benchflow-ai/skillsbench/archive/${ref}.tar.gz"
# Task envs are heterogeneous — some have curl, some only wget, some only python3.
if command -v curl >/dev/null 2>&1; then curl -fsSL "$url" -o "$tmp"
elif command -v wget >/dev/null 2>&1; then wget -qO "$tmp" "$url"
else python3 -c 'import sys,urllib.request; open(sys.argv[2],"wb").write(urllib.request.urlopen(sys.argv[1]).read())' "$url" "$tmp"
fi
tar -xzf "$tmp" -C /solution --strip-components=4 "skillsbench-${ref}/tasks/${task}/solution"
rm -f "$tmp"

cd /root
exec bash /solution/solve.sh
