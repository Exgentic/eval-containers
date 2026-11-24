"""Grade one AssertLLM2-verify task: the agent's verdict against the task's label.

usage: grade.py <task dir> <answer file>

The label is assumed from where the variant came from (golden -> correct, either
kind of mutant -> incorrect); see README.md. Reward is 1.0 for a recognized
verdict that matches it, else 0.0. The explanation is kept for people to read
and never changes the reward.
"""

import json
import sys
from pathlib import Path

task, answer_file = Path(sys.argv[1]), Path(sys.argv[2])
label = (task / "label.txt").read_text().strip()
kind = (task / "variant_kind.txt").read_text().strip()

raw = answer_file.read_text(errors="replace") if answer_file.is_file() else None
try:
    answer = json.loads(raw) if raw is not None else None
except ValueError:
    answer = None
verdict = answer.get("verdict") if isinstance(answer, dict) else None
if verdict not in ("correct", "incorrect"):
    verdict = None
explanation = answer.get("explanation") if isinstance(answer, dict) else None

report = {
    "label": label,
    "variant_kind": kind,
    "verdict": verdict,
    "answer_valid": verdict is not None,
    "verdict_correct": verdict == label,
    "explanation_present": isinstance(explanation, str) and bool(explanation.strip()),
}
if raw is None:
    report["error"] = "no /home/agent/answer.json that the agent can read"
elif verdict is None:
    report["error"] = (
        'answer is not a JSON object with "verdict" "correct" or "incorrect"'
    )

out = Path("/output/task")
out.mkdir(parents=True, exist_ok=True)
if raw is not None:
    (out / "answer.json").write_text(raw)
(out / "verifier_report.json").write_text(json.dumps(report, indent=2) + "\n")
Path("/logs/verifier/reward.txt").write_text(
    "1.0" if report["verdict_correct"] else "0.0"
)
print(json.dumps(report))
