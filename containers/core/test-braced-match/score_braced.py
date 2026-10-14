"""Shared ZeroBench-style scorer — the single source of truth for both the
model-only (native) harness and the agent-facing grader.

Logic: extract the LAST ``{...}`` from the text (upstream regex ``\\{(.*?)\\}``)
and match it case-insensitively to the gold answer.

Two consumers, one implementation:
  - import  → the native harness (``run_zerobench.py``) calls ``extract_answer`` /
    ``is_correct`` per sample.
  - CLI     → ``test.sh`` (the agent-facing ``/grade.sh``) runs this file: it
    reads the agent's stdout, writes a fail-closed ``/logs/verifier/reward.txt``.
"""

import re

_BRACE = re.compile(r"\{(.*?)\}", re.DOTALL)


def extract_answer(text):
    """Last {...} in the text, else "" (no braces → unanswered → wrong)."""
    if not text:
        return ""
    m = _BRACE.findall(text)
    return m[-1].strip() if m else ""


def is_correct(parsed, gold):
    """Case-insensitive normalized equality (upstream: lower-cased, leading+length)."""
    return bool(parsed) and parsed.strip().lower() == (gold or "").strip().lower()


if __name__ == "__main__":
    import os
    import sys

    os.makedirs("/logs/verifier", exist_ok=True)
    with open("/logs/verifier/reward.txt", "w") as f:
        f.write("0")  # fail-closed baseline

    gold = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("EXPECTED_ANSWER", "")
    text = ""
    stdout = "/output/agent/stdout.log"
    if os.path.exists(stdout):
        with open(stdout, encoding="utf-8", errors="replace") as f:
            text = f.read()

    parsed = extract_answer(text)
    reward = 1.0 if is_correct(parsed, gold) else 0.0
    with open("/logs/verifier/reward.txt", "w") as f:
        f.write(repr(reward))
    print(f"[score-braced] gold={gold!r} parsed={parsed[:80]!r} reward={reward}")
