#!/bin/bash
# Oracle for aider-polyglot: install Exercism's own reference implementation into
# the agent workspace, then let the real grader run. The reference ships under
# .meta/ inside the exercise tree, which is mode 600 — readable here because the
# oracle runs as root in place of the agent, never by the agent itself.
#
# .meta/config.json lists example files positionally against the editable solution
# files (rust: example.rs -> src/lib.rs, with Cargo.toml having no counterpart;
# cpp: .cpp and .h in order). A language that ships more reference files than
# editable ones (java helper classes) drops the extras beside the first one.
set -euo pipefail
TASK_DIR="/tasks/${EVAL_TASK_ID:-0}"
python3 - "$(cat "$TASK_DIR/exercise_dir.txt")" <<'PY'
import json, pathlib, shutil, sys

exercise = pathlib.Path(sys.argv[1])
files = json.loads((exercise / ".meta/config.json").read_text())["files"]
solution, example = files["solution"], files.get("example", [])
if not example:
    raise SystemExit(f"{exercise}: no reference solution in .meta/config.json")

for i, src in enumerate(example):
    if i < len(solution):
        rel = solution[i]
    else:
        rel = str(pathlib.PurePath(solution[0]).parent / pathlib.PurePath(src).name)
    dst = pathlib.Path("/app") / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(exercise / src, dst)
    print(f"{src} -> {dst}")
PY
