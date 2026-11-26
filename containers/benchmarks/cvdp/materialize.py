"""Split a pinned CVDP non-agentic JSONL into flat per-task files (rules 15/16).

    /tasks/<n>/id.txt        upstream datapoint id (e.g. cvdp_copilot_16qam_mapper_0001)
    /tasks/<n>/task.json     {id, category, prompt, input_context, output_files}
    /tasks/<n>/harness/...   hidden harness: docker-compose.yml, src/* -> /src at grade
    /tasks/<n>/output/...    reference solution files, written only when non-empty

<n> is the row's 0-based line index (sequential ids, rule 15); upstream rows are
sorted by id, so the index is stable for a pinned revision. The public release
redacts `output.context` (the reference solution) to empty strings; Si2 LBC
members receive it.
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
with open(src) as f:
    for n, line in enumerate(ln for ln in f if ln.strip()):
        row = json.loads(line)
        base = os.path.join(out, str(n))
        write(os.path.join(base, "id.txt"), row["id"])
        write(
            os.path.join(base, "task.json"),
            json.dumps(
                {
                    "id": row["id"],
                    "category": int(row["categories"][0][3:]),
                    "difficulty": row["categories"][1]
                    if len(row["categories"]) > 1
                    else "",
                    "prompt": row["input"]["prompt"],
                    "input_context": row["input"]["context"],
                    "output_files": list(row["output"]["context"].keys()),
                }
            ),
        )
        for rel, body in row["harness"]["files"].items():
            write(os.path.join(base, "harness", rel), body)
        for rel, body in row["output"]["context"].items():
            if body:
                write(os.path.join(base, "output", rel), body)

count = n + 1
print(f"materialized {count} tasks")
expected = int(os.environ.get("CVDP_EXPECTED_TASKS", "302"))
assert count == expected, f"expected {expected} tasks, got {count}"
