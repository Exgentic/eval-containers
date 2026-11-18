#!/bin/bash
# Verifier: the agent's verdict in /home/agent/answer.json against the hidden
# label. The answer and the verdict's details go to /output/task/.
mkdir -p /logs/verifier /output/task
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)

# Read the agent's file as the agent, so a symlink into /tasks reads nothing.
answer=$(mktemp)
trap 'rm -f "$answer"' EXIT
if setpriv --reuid=1002 --regid=1002 --clear-groups \
     cat /home/agent/answer.json > "$answer" 2>/dev/null; then
  python3 /tests/grade.py "/tasks/$EVAL_TASK_ID" "$answer"
else
  python3 /tests/grade.py "/tasks/$EVAL_TASK_ID" /nonexistent
fi
