#!/bin/bash
# Verifier: run swebench's eval script on the tree the agent left, then score its
# log with swebench's report (1 = resolved, 0 = not).
mkdir -p /logs/verifier /output/agent
cd /testbed || exit 1
git -c core.fileMode=false diff "$(cat /tasks/0/base_commit.txt)" > /output/agent/patch.diff 2>/dev/null
# The log goes to /tmp, not /logs: the script prints whole commits, the largest
# log seen is 49 MB, and /logs holds 100 MB — a truncated log grades wrong.
# 1800 s is the harness's own test timeout, and it counts a timeout as
# unresolved: some openlayers suites never return on a tree without the fix.
timeout -k 30 1800 bash /tasks/0/eval.sh > /tmp/test_output.log 2>&1
/tests/python/bin/python3 /tests/grade.py /tmp/test_output.log > /logs/verifier/reward.txt 2>/logs/verifier/grade.log \
  || echo 0 > /logs/verifier/reward.txt
