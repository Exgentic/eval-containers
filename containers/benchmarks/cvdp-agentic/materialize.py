"""Split a pinned CVDP agentic JSONL into flat per-task files (rules 15/16).

    /tasks/<n>/id.txt        upstream datapoint id (e.g. cvdp_agentic_64b66b_codec_0001)
    /tasks/<n>/problem.txt   upstream `prompt` (what upstream writes to /code/prompt.json)
    /tasks/<n>/context/...   agent workspace files (docs/, rtl/, verif/) -> /code/...
    /tasks/<n>/harness/...   hidden harness: docker-compose.yml, src/* -> /src at grade

<n> is the row's 0-based line index (sequential ids, rule 15); upstream rows are
already sorted by id, so the index is stable for a pinned revision.

    /tasks/<n>/patch/...     reference solution hunks, written only when non-empty
"""

import json
import os
import sys

src, out = sys.argv[1], sys.argv[2]


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


n = 0
with open(src) as f:
    for n, line in enumerate(ln for ln in f if ln.strip()):
        row = json.loads(line)
        base = os.path.join(out, str(n))
        write(os.path.join(base, "id.txt"), row["id"])
        write(os.path.join(base, "problem.txt"), row["prompt"])
        write(os.path.join(base, "categories.txt"), ",".join(row.get("categories", [])))
        for rel, body in row["context"].items():
            write(os.path.join(base, "context", rel), body)
        for rel, body in row["harness"].items():
            write(os.path.join(base, "harness", rel), body)
        # Reference solution, only where the data carries one (the public release
        # redacts it; Si2 LBC members receive it). Root-only like the rest.
        for rel, body in (row.get("patch") or {}).items():
            if body:
                write(os.path.join(base, "patch", rel), body)

count = n + 1
print(f"materialized {count} tasks")
expected = int(os.environ.get("CVDP_EXPECTED_TASKS", "92"))
assert count == expected, f"expected {expected} tasks, got {count}"
