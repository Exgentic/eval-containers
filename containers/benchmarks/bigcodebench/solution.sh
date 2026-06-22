#!/bin/bash
# Oracle for BigCodeBench: emit a complete canonical program = code_prompt (imports
# + signature) + canonical_solution (body), both from the same pinned dataset row.
# Both fields are baked root-only into /tasks by the duckdb loader; oracle runs as
# root so it can read them here, while the non-root agent cannot (no gold leak).
# No network/pyarrow needed. Output -> stdout.
set -euo pipefail
d="/tasks/${EVAL_TASK_ID:-0}"
cat "$d/code_prompt.txt" "$d/canonical_solution.txt"
