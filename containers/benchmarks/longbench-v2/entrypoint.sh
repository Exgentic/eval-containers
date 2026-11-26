#!/bin/bash
# The rendered prompt (up to ~16 MB) cannot travel in the TASK env var, so the
# agent reads it from /app/task.txt; TASK points there.
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  cp "/tasks/$EVAL_TASK_ID/problem.txt" /app/task.txt
  chmod 644 /app/task.txt
  export TASK="Your task is in /app/task.txt: a long text followed by a multiple-choice question at the end of the file. Read it carefully and answer exactly in the format it asks for."
  EXPECTED_ANSWER=$(cat "/tasks/$EVAL_TASK_ID/answer.txt")
  export EXPECTED_ANSWER
fi
exec "$@"
