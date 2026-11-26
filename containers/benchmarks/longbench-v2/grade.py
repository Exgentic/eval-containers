"""LongBench v2 grader: upstream's extract_answer + judge (THUDM/LongBench pred.py).

Drop every '*', take "The correct answer is (X)", else "The correct answer is X";
reward 1 iff that letter equals the gold letter. No match, or no stdout, is wrong.

One deviation: upstream reads a single short reply and takes the FIRST match.
Here stdout can be an agent's whole transcript (interim answers, tool output,
echoed context — task 134's text itself contains "The correct answer is D"), so
the LAST match of each pattern is taken: the agent's final answer. On a single
reply that states one answer, the two agree.
"""

import os
import re


def extract_answer(response):
    response = response.replace("*", "")
    for pattern in (
        r"The correct answer is \(([A-D])\)",
        r"The correct answer is ([A-D])",
    ):
        matches = re.findall(pattern, response)
        if matches:
            return matches[-1]
    return None


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
