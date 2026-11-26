#!/bin/bash
# Oracle gold: grade a reply in upstream's required format carrying the task's
# gold letter, through the harness's own extract_answer + judge (no model call).
# The no-op leaves /grade.sh's fail-closed 0. Runs as root; /tasks is root-only.
set -euo pipefail
python3 /app/run_longbench_v2.py --score \
  "The correct answer is ($(cat "/tasks/${EVAL_TASK_ID:-0}/answer.txt"))"
