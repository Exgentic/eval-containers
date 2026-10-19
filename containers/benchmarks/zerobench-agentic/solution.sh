#!/bin/bash
# Oracle for zerobench-agentic: emit the gold answer behind the `FINAL ANSWER:`
# sentinel the transcript extractor anchors to. The default
# `printf '%s' "$EXPECTED_ANSWER"` emits a bare answer with no sentinel line,
# which extract_sentinel() maps to "" (fail-closed), scoring 0.0 on a correct
# answer. The sentinel must start a line, and the grader takes the LAST such
# line, so a single trailing line is exactly the gold shape.
set -euo pipefail
printf 'FINAL ANSWER: %s\n' "$(cat "/tasks/${EVAL_TASK_ID:-0}/answer.txt")"
