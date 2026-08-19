#!/usr/bin/env python3
"""Build-time task staging for HANDBOOK.md.

Maps the upstream per-task directories (tasks/<name>/) into the flat, integer-
addressed seed corpus the runtime materializes from (rule 15/16):

    /root/tasks/<int>/
        instruction.md                  # the user request
        system_prompt.md                # persona / date / tool families
        rubrics.json                     # gold verifiers (root-only at runtime)
        initial_workspace/               # company docs + the handbook
        initial_external_services/       # per-service JSON seeds
        id.txt                           # upstream task name (rule 15)

Ordering is the sorted upstream name, so integer ids are stable across builds
(rule 3, reproducible-by-default). Also writes tasks.txt (int<TAB>name) as a
human-facing map. Runs inside `docker build` (see Dockerfile); the whole
clone→stage→cleanup happens in one RUN so the gold never lands in its own layer.

Usage: stage-tasks.py <upstream-tasks-dir> <dest-root>
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

EXPECTED_TASKS = 65


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: stage-tasks.py <upstream-tasks-dir> <dest-root>", file=sys.stderr)
        return 2
    src_root = Path(sys.argv[1])
    dest_root = Path(sys.argv[2])

    names = sorted(p.name for p in src_root.iterdir() if p.is_dir())
    if not names:
        print(f"stage-tasks: no task dirs under {src_root}", file=sys.stderr)
        return 1

    dest_root.mkdir(parents=True, exist_ok=True)
    manifest = []
    for idx, name in enumerate(names):
        src = src_root / name
        dst = dest_root / str(idx)
        dst.mkdir(parents=True, exist_ok=True)

        for fname in ("instruction.md", "system_prompt.md"):
            fsrc = src / fname
            if fsrc.is_file():
                shutil.copy2(fsrc, dst / fname)

        rubrics = src / "tests" / "rubrics.json"
        if not rubrics.is_file():
            print(f"stage-tasks: {name} has no tests/rubrics.json", file=sys.stderr)
            return 1
        shutil.copy2(rubrics, dst / "rubrics.json")

        env = src / "environment"
        for sub in ("initial_workspace", "initial_external_services"):
            ssrc = env / sub
            if ssrc.is_dir():
                shutil.copytree(ssrc, dst / sub, dirs_exist_ok=True)

        (dst / "id.txt").write_text(name + "\n", encoding="utf-8")
        manifest.append(f"{idx}\t{name}")

    (dest_root / "tasks.txt").write_text("\n".join(manifest) + "\n", encoding="utf-8")

    count = len(names)
    print(f"stage-tasks: staged {count} tasks into {dest_root}")
    if count != EXPECTED_TASKS:
        print(
            f"stage-tasks: WARNING expected {EXPECTED_TASKS} tasks at the pinned "
            f"revision, staged {count}",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
