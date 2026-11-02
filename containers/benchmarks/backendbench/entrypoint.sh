#!/bin/bash
# Without a GPU the task cannot be graded, so stop before an agent spends a run
# on it — a missing device must fail loudly, never score as a wrong kernel.
if ! nvidia-smi -L >/dev/null 2>&1; then
  echo "backendbench: no NVIDIA GPU is visible — run it with one (compose reserves it; on k8s the preset requests nvidia.com/gpu)" >&2
  exit 1
fi
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  chown -R 1002 /app 2>/dev/null || true
  TASK=$(cat "/tasks/$EVAL_TASK_ID/problem.txt")
  export TASK
fi
exec "$@"
