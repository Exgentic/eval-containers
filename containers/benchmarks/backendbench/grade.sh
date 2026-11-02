#!/bin/bash
# Verifier: upstream's CLI on /app/solution.py as the operator's kernel, run as the
# grader uid, turned into the reward. What was graded and the verdict go to /output/task/.
mkdir -p /logs/verifier /output/task
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)

# The agent is done; anything it left running would share the GPU with the grader.
pkill -9 -u 1002 2>/dev/null

WORK=$(mktemp -d /work/run.XXXXXX) || exit 1
trap 'rm -rf "$WORK"' EXIT
op=$(cat "/tasks/$EVAL_TASK_ID/id.txt")
folder=${op//./__}
# Upstream's directory backend loads <folder>/<folder>_implementation*.py.
mkdir -p "$WORK/kernels/$folder" "$WORK/tmp"
# Read the agent's file as the agent, so a symlink into /tasks or /tests reads nothing.
setpriv --reuid=1002 --regid=1002 --clear-groups cat /app/solution.py \
  > "$WORK/kernels/$folder/${folder}_implementation_v1.py" 2>/dev/null
cp "$WORK/kernels/$folder/${folder}_implementation_v1.py" /output/task/solution.py
chown -R 1004:1004 "$WORK"

# Triton's cache goes in $WORK too: on k8s /tmp is a 1 GiB in-memory volume.
cd "$WORK" && HOME="$WORK" TMPDIR="$WORK/tmp" TRITON_CACHE_DIR="$WORK/tmp/triton" \
  PYTHONPATH=/tests/deps timeout 900 setpriv --reuid=1004 --regid=1004 --clear-groups \
  python3 -m BackendBench.scripts.main --suite opinfo --backend directory --ops "$op" \
  --ops-directory "$WORK/kernels" --log-dir "$WORK/out" > /output/task/backendbench.log 2>&1
rc=$?
# The agent's code ran as 1004 and may have forked past the timeout.
pkill -9 -u 1004 2>/dev/null
find /tmp /dev/shm -uid 1004 -delete 2>/dev/null

python3 /tests/grade.py "$WORK" "$op" >> /output/task/backendbench.log 2>&1
cp "$WORK/result.json" /output/task/backendbench.json 2>/dev/null
cp "$WORK/out/full_results.json" "$WORK/out/operator_summary.csv" /output/task/ 2>/dev/null
jq -r 'if .reward == 1 then "1.0" else "0.0" end' "$WORK/result.json" \
  > /logs/verifier/reward.txt 2>/dev/null || echo 0.0 > /logs/verifier/reward.txt
exit $rc
