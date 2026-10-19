#!/bin/bash
# Verifier: upstream's harness on the agent's diff, beside the expert's patch.
# What was graded and the verdict go to /output/task/.
mkdir -p /logs/verifier /output/task
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)

pkill -9 -u agent 2>/dev/null

# The prediction, as upstream's inference harness extracts it.
cd /testbed || exit 1
git config --global --add safe.directory /testbed
git add -N . && git diff --binary "$(cat /tests/head)" > /output/task/patch.diff

WORK=$(mktemp -d /tmp/grade.XXXXXX) && cd "$WORK" || exit 1
timeout 28800 /opt/miniconda3/bin/python /tests/grade.py "$WORK" /output/task/patch.diff \
  > /output/task/swefficiency.log 2>&1
rc=$?
cp "$WORK/result.json" /output/task/swefficiency.json 2>/dev/null
cp -r "$WORK/logs" /output/task/ 2>/dev/null
jq -r '.reward' "$WORK/result.json" > /logs/verifier/reward.txt 2>/dev/null \
  || echo 0.0 > /logs/verifier/reward.txt
exit $rc
