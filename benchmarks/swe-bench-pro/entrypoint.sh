#!/bin/bash
# Hand the repo to the agent and seed TASK from the problem statement.
chown -R agent:agent /app 2>/dev/null || true
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK="Fix this GitHub issue in the repository at /app. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
