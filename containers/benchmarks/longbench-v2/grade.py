"""LongBench v2 grader: upstream's extract_answer + judge (THUDM/LongBench pred.py).

Drop every '*', take the first "The correct answer is (X)", else the first
"The correct answer is X"; reward 1 iff that letter equals the gold letter.
No match, or no stdout, is wrong.
"""

import os
import re


def extract_answer(response):
    response = response.replace("*", "")
    match = re.search(r"The correct answer is \(([A-D])\)", response)
    if match:
        return match.group(1)
    match = re.search(r"The correct answer is ([A-D])", response)
    return match.group(1) if match else None


try:
    with open("/output/agent/stdout.log", encoding="utf-8", errors="replace") as f:
        response = f.read()
except FileNotFoundError:
    response = ""
gold = os.environ.get("EXPECTED_ANSWER", "").strip()
pred = extract_answer(response)
with open("/logs/verifier/reward.txt", "w") as f:
    f.write("1\n" if gold and pred == gold else "0\n")
print(f"[score] gold={gold!r} pred={pred!r}")
