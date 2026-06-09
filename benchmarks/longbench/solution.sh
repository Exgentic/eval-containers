#!/bin/bash
# Oracle: emit the gold answer file VERBATIM. The grader is byte-sensitive (raw
# read / member-of answers list), so the default `printf "%s" "$EXPECTED_ANSWER"`
# — which strips the trailing newline via $(cat …) — would fall short. The
# oracle redirects this stdout to /output/agent/stdout.log.
set -euo pipefail
cat "/tasks/${EVAL_TASK_ID}/answer.txt"
