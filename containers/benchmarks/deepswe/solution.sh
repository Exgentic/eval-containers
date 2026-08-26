#!/bin/bash
# Oracle gold for DeepSWE v1.1 — fetch THIS task's upstream reference solution at
# the pinned ref and apply it in /app; the grader then runs the hidden tests
# against the result (gold => reward 1, no-op => 0).
#
# Fetched fresh at oracle run time, never baked into the image (rules 9, 20a): the
# gold is DERIVED from upstream's own reference patch, not a literal answer copied
# into the repo, so it stays valid as the pinned revision moves. The oracle runs
# as root with the network the sandboxed agent lacks. See core/oracle.
#
# Upstream ships gold as solution/solve.sh (the driver) + solution/solution.patch
# (the diff), where solve.sh reads /solution/solution.patch — so both are staged
# there. solve.sh also commits the work, which is what the verifier grades.
set -euo pipefail

ref="${DEEPSWE_REF:?DEEPSWE_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the real
# task name baked into the image (DEEPSWE_TASK), falling back to EVAL_TASK_ID (rule 24i).
task="${DEEPSWE_TASK:-${EVAL_TASK_ID:?DEEPSWE_TASK/EVAL_TASK_ID not set}}"
base="https://raw.githubusercontent.com/datacurve-ai/deep-swe/${ref}/tasks/${task}/solution"

fetch() {
  if command -v curl >/dev/null 2>&1; then curl -fsSL "$1"
  elif command -v wget >/dev/null 2>&1; then wget -qO- "$1"
  else python3 -c 'import sys,urllib.request; sys.stdout.write(urllib.request.urlopen(sys.argv[1]).read().decode())' "$1"
  fi
}

mkdir -p /solution
fetch "${base}/solution.patch" > /solution/solution.patch
fetch "${base}/solve.sh"       > /solution/solve.sh

cd /app
bash /solution/solve.sh
