#!/bin/bash
# Save the repository and its conda env for grading, then hand both to the agent,
# who rebuilds the package into that env (README.md).
if [ ! -d /tests/pristine/testbed ]; then
  cp -a /testbed /tests/pristine/testbed
  cp -a /opt/miniconda3/envs/testbed /tests/pristine/env
  chown -R agent:agent /testbed /opt/miniconda3/envs/testbed 2>/dev/null || true
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK=$(cat /tasks/0/problem.txt)
  export TASK
fi
exec "$@"
