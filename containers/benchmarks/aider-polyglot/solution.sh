#!/bin/bash
# Oracle for aider-polyglot: install Exercism's own reference implementation into
# the agent workspace, then let the real grader run. The reference ships under
# .meta/ inside the exercise tree, which is mode 600 — readable here because the
# oracle runs as root in place of the agent, never by the agent itself.
#
# Reference and editable files are matched by extension, not by position: cpp
# exercises whose reference is only `.meta/example.h` list both `<ex>.cpp` and
# `<ex>.h` as editable, so pairing by index writes a header into the .cpp slot
# and the build fails on the reference solution itself.
#
# The editable set excludes the build manifests (Cargo.toml, CMakeLists.txt),
# matching upstream, and grading takes only editable files out of /app — so a
# reference spanning more files than are editable cannot be installed at all.
# That is a task unfit for the oracle list, not something to paper over.
set -euo pipefail
TASK_DIR="/tasks/${EVAL_TASK_ID:-0}"
python3 - "$(cat "$TASK_DIR/exercise_dir.txt")" "$(cat "$TASK_DIR/solution_files.txt")" <<'PY'
import json, pathlib, shutil, sys

exercise = pathlib.Path(sys.argv[1])
solution = [f for f in sys.argv[2].split("\n") if f]
example = json.loads((exercise / ".meta/config.json").read_text())["files"].get("example", [])
if not example:
    raise SystemExit(f"{exercise}: no reference solution in .meta/config.json")

unclaimed = list(solution)
for src in example:
    suffix = pathlib.PurePath(src).suffix
    match = next((f for f in unclaimed if pathlib.PurePath(f).suffix == suffix), None)
    if match is None:
        raise SystemExit(f"{exercise}: reference {src} has no editable file to hold it")
    unclaimed.remove(match)
    dst = pathlib.Path("/app") / match
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(exercise / src, dst)
    print(f"{src} -> {dst}")
PY
