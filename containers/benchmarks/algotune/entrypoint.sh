#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  chown -R 1002 /app 2>/dev/null || true
  TASK=$(cat "/tasks/$EVAL_TASK_ID/problem.txt")
  export TASK
fi
exec "$@"
