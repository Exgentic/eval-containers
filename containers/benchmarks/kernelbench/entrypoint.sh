#!/bin/bash
# Without a GPU the task cannot be graded, so stop before an agent spends a run
# on it — a missing device must fail loudly, never score as a wrong kernel.
if ! nvidia-smi -L >/dev/null 2>&1; then
  echo "kernelbench: no NVIDIA GPU is visible — run it with one (compose reserves it; on k8s the preset requests nvidia.com/gpu)" >&2
  exit 1
fi
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK_DIR="/tasks/$EVAL_TASK_ID"
  cp "$TASK_DIR/reference.txt" /app/reference.py
  chown -R 1002 /app 2>/dev/null || true
  TASK=$(cat "$TASK_DIR/problem.txt")
  export TASK
fi
exec "$@"
