#!/bin/bash
# Verifier: upstream AlgoTune's own evaluator on /app/solver.py, run as the grader uid,
# turned into the reward. What was graded and the verdict go to /output/task/.
mkdir -p /logs/verifier /output/task
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)

pkill -9 -u 1002 2>/dev/null

WORK=$(mktemp -d /work/run.XXXXXX) || exit 1
trap 'rm -rf "$WORK"' EXIT
mkdir "$WORK/solver"
# Read the agent's file as the agent, so a symlink into /tasks or /tests reads nothing.
setpriv --reuid=1002 --regid=1002 --clear-groups cat /app/solver.py > "$WORK/solver/solver.py" 2>/dev/null
cp "$WORK/solver/solver.py" /output/task/solver.py
# Upstream's evaluator writes into its own task directory, so it runs from a copy.
cp -r /tests/upstream "$WORK/"
mkdir "$WORK/tmp" && chown -R 1004:1004 "$WORK"

HOME="$WORK" TMPDIR="$WORK/tmp" PYTHONPATH="$WORK/upstream" timeout 7200 \
  setpriv --reuid=1004 --regid=1004 --clear-groups python3 /tests/grade.py "$WORK" \
  "$(cat "/tasks/$EVAL_TASK_ID/id.txt")" "$(cat "/tasks/$EVAL_TASK_ID/n.txt")" \
  > /output/task/algotune.log 2>&1
rc=$?
pkill -9 -u 1004 2>/dev/null
find /tmp /dev/shm -uid 1004 -delete 2>/dev/null

cp "$WORK/result.json" /output/task/algotune.json 2>/dev/null
jq -r '.reward' "$WORK/result.json" > /logs/verifier/reward.txt 2>/dev/null \
  || echo 0.0 > /logs/verifier/reward.txt
exit $rc
