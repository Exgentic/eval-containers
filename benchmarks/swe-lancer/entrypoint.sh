#!/bin/bash
# Hand the Expensify checkout to the agent and seed TASK from the issue prompt.
chown -R agent:agent /app/expensify 2>/dev/null || true
if [ -n "${EVAL_TASK_ID:-}" ] && [ -z "${TASK:-}" ]; then
  TASK="You are fixing a freelance software-engineering task in the Expensify codebase at /app/expensify. Do NOT modify test files. When you are done, your change must pass the upstream end-to-end tests.

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
