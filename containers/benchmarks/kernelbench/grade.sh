#!/bin/bash
# Verifier: upstream's evaluator on /app/solution.py, run as the grader uid,
# turned into the reward. What was graded and the verdict go to /output/task/.
mkdir -p /logs/verifier /output/task
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)

# The agent is done; anything it left running would share the GPU with the timing.
pkill -9 -u 1002 2>/dev/null

WORK=$(mktemp -d /work/run.XXXXXX) || exit 1
trap 'rm -rf "$WORK"' EXIT
cp "/tasks/$EVAL_TASK_ID/reference.txt" "$WORK/reference.py"
# Read the agent's file as the agent, so a symlink into /tasks or /tests reads nothing.
setpriv --reuid=1002 --regid=1002 --clear-groups cat /app/solution.py > "$WORK/solution.py" 2>/dev/null
cp "$WORK/solution.py" /output/task/solution.py
mkdir "$WORK/tmp" && chown -R 1004:1004 "$WORK"

# nvcc's scratch goes in $WORK too: on k8s /tmp is a 1 GiB in-memory volume.
HOME="$WORK" TMPDIR="$WORK/tmp" PYTHONPATH=/tests/deps timeout 900 \
  setpriv --reuid=1004 --regid=1004 --clear-groups python3 /tests/grade.py "$WORK" \
  > /output/task/kernelbench.log 2>&1
rc=$?
# The agent's code ran as 1004 and may have forked past the timeout.
pkill -9 -u 1004 2>/dev/null
find /tmp /dev/shm -uid 1004 -delete 2>/dev/null

cp "$WORK/result.json" /output/task/kernelbench.json 2>/dev/null
jq -r 'if .reward == 1 then "1.0" else "0.0" end' "$WORK/result.json" \
  > /logs/verifier/reward.txt 2>/dev/null || echo 0.0 > /logs/verifier/reward.txt
exit $rc
