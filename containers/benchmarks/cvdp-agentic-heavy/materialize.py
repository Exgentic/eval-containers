"""Split a pinned CVDP agentic-heavy JSONL into flat per-task files (rules 15/16).

    /tasks/<n>/id.txt          upstream datapoint id (e.g. cvdp_agentic_heavy_rvx_0004)
    /tasks/<n>/problem.txt     upstream `prompt` (what upstream writes to /code/prompt.json)
    /tasks/<n>/repo.txt        context repo = git bundle name (cvdp_agentic_heavy_<project>)
    /tasks/<n>/commit.txt      sanitized context commit to check out
    /tasks/<n>/root.txt        subtree copied to /code: `external` or `.` (upstream rule below)
    /tasks/<n>/harness/...     hidden harness: docker-compose.yml, src/* -> /src at grade
    /tasks/<n>/patch/...       reference solution hunks, written only when non-empty

<n> is the row's 0-based line index (sequential ids, rule 15). The workspace rule
is upstream's (src/dataset_processor.py create_repo, heavy branch): copy
`external/` of the context checkout for CVDP-hosted repos, the whole checkout for
`_ext_` repos ("ARE the external code at the repo root").
"""

import json
import os
import sys

src, out = sys.argv[1], sys.argv[2]


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


n = -1
repos = set()
with open(src) as f:
    for n, line in enumerate(ln for ln in f if ln.strip()):
        row = json.loads(line)
        base = os.path.join(out, str(n))
        repo = row["context"]["repo"]
        repos.add(repo)
        write(os.path.join(base, "id.txt"), row["id"])
        write(os.path.join(base, "problem.txt"), row["prompt"])
        write(os.path.join(base, "categories.txt"), ",".join(row.get("categories", [])))
        write(os.path.join(base, "repo.txt"), repo)
        write(os.path.join(base, "commit.txt"), row["context"]["commit"])
        write(os.path.join(base, "root.txt"), "." if "_ext_" in repo else "external")
        for rel, body in row["harness"].items():
            if body is None:  # stale __pycache__/*.pyc entries; upstream skips None too
                continue
            write(os.path.join(base, "harness", rel), body)
        for rel, body in (row.get("patch") or {}).items():
            if body:
                write(os.path.join(base, "patch", rel), body)

count = n + 1
write(os.path.join(out, "repos.txt"), "\n".join(sorted(repos)) + "\n")
print(f"materialized {count} tasks over {len(repos)} repos")
expected = int(os.environ.get("CVDP_EXPECTED_TASKS", "75"))
assert count == expected, f"expected {expected} tasks, got {count}"
