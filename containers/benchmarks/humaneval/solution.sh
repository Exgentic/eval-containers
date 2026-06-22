#!/bin/bash
# Oracle for humaneval: emit the upstream canonical_solution (the function body).
# The grader concatenates it after the prompt (signature + docstring) and runs the
# unit tests. canonical_solution is baked root-only into /tasks by the duckdb loader
# (from the pinned parquet); oracle runs as root so it can read it here, while the
# non-root agent cannot (no gold leak). No network/pyarrow needed. Output -> stdout.
set -euo pipefail
cat "/tasks/${EVAL_TASK_ID:-0}/canonical_solution.txt"
