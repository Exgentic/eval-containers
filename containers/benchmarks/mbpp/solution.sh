#!/bin/bash
# Oracle for mbpp: emit the canonical MBPP `code`. The grader runs it against the
# task's asserts. `code` is baked root-only into /tasks by the duckdb loader (from
# the pinned parquet); oracle runs as root so it can read it here, while the
# non-root agent cannot (no gold leak). No network/pyarrow needed. Output -> stdout.
set -euo pipefail
cat "/tasks/${EVAL_TASK_ID:-0}/code.txt"
