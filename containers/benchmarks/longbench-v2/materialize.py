"""Materialize LongBench v2 into /tasks/all.jsonl (one row per task, dataset order).

Usage: materialize.py <dataset revision>
"""

import json
import os
import sys
import urllib.request

rev = sys.argv[1]
urllib.request.urlretrieve(
    f"https://huggingface.co/datasets/zai-org/LongBench-v2/resolve/{rev}/data.json",
    "/tmp/data.json",
)
data = json.load(open("/tmp/data.json", encoding="utf-8"))
os.remove("/tmp/data.json")

# Verbatim prompts/0shot.txt (THUDM/LongBench@2e00731), no trailing newline.
TEMPLATE = """Please read the following text and answer the question below.

<text>
$DOC$
</text>

What is the correct answer to this question: $Q$
Choices:
(A) $C_A$
(B) $C_B$
(C) $C_C$
(D) $C_D$

Format your response as follows: "The correct answer is (insert answer here)"."""

os.makedirs("/tasks", exist_ok=True)
with open("/tasks/all.jsonl", "w", encoding="utf-8") as out:
    for item in data:
        # Same substitution order as upstream pred.py.
        prompt = (
            TEMPLATE.replace("$DOC$", item["context"].strip())
            .replace("$Q$", item["question"].strip())
            .replace("$C_A$", item["choice_A"].strip())
            .replace("$C_B$", item["choice_B"].strip())
            .replace("$C_C$", item["choice_C"].strip())
            .replace("$C_D$", item["choice_D"].strip())
        )
        assert item["answer"] in "ABCD" and len(item["answer"]) == 1, item["_id"]
        row = {"id": item["_id"], "problem": prompt, "answer": item["answer"]}
        # The leaderboard's Easy/Hard and Short/Medium/Long splits (result.py).
        for k in ("domain", "sub_domain", "difficulty", "length"):
            row[k] = item[k]
        out.write(json.dumps(row) + "\n")
# The pinned revision's shape (rule 3): 503 questions, unique upstream ids, and
# every gold letter names a non-empty choice.
assert len(data) == 503, f"expected 503 LongBench v2 tasks, got {len(data)}"
assert len({item["_id"] for item in data}) == 503, "duplicate _id"
assert all(
    item[f"choice_{item['answer']}"].strip() for item in data
), "empty gold choice"
print(f"Wrote {len(data)} LongBench v2 tasks")
