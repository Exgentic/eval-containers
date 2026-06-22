#!/bin/bash
# Oracle for humanevalplus: emit the canonical_solution (function body). The grader
# splices it onto the prompt + EvalPlus harness and runs check(entry_point).
# canonical_solution is baked root-only into /tasks by the duckdb loader (from the
# pinned parquet); oracle runs as root so it can read it here, while the non-root
# agent cannot (no gold leak). HumanEval/32 needs a hand-crafted completion (genuine
# EvalPlus grader edge case). No network/pyarrow needed. Output -> stdout.
set -euo pipefail
d="/tasks/${EVAL_TASK_ID:-0}"
if [ "$(cat "$d/id.txt" 2>/dev/null)" = "HumanEval/32" ]; then
  printf '\n    return ([],)\n'
else
  cat "$d/canonical_solution.txt"
fi
