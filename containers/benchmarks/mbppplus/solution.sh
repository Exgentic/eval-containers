#!/bin/bash
# Oracle for MBPP+ (evalplus/mbppplus): emit the canonical `code`. The grader
# splices it into the EvalPlus harness and runs python3. `code` is baked root-only
# into /tasks by the duckdb loader (from the pinned parquet); oracle runs as root so
# it can read it here, while the non-root agent cannot (no gold leak). Output -> stdout.
set -euo pipefail
cat "/tasks/${EVAL_TASK_ID:-0}/code.txt"
