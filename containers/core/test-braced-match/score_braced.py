"""Shared ZeroBench scorer. The MATCH is shared; the EXTRACTION is per-surface,
because the two surfaces have different output shapes:

  - Native (model-only): stdout is a SINGLE model completion. Upstream's protocol
    — "give the final answer in curly braces {final answer}" — is correct here,
    because the completion ends when the model stops, so the last ``{...}`` is the
    answer. ``extract_answer`` implements it; the native harness imports it.

  - Agent (tool-using): stdout is a whole TRANSCRIPT — tool invocations, command
    echoes, results. Braces are ordinary characters there (code, JSON, dict
    literals), and the harness can append tool output AFTER the agent's final
    message. So "last ``{...}`` in the text" is NOT the answer — it silently
    breaks whenever the agent does ordinary work. The agent instead emits a
    DELIBERATE sentinel line (``FINAL ANSWER: <answer>``); ``extract_sentinel``
    anchors to the LAST such line, which trailing tool output does not disturb.
    The CLI (``test.sh`` → the agent's ``/grade.sh``) uses this path.

``is_correct`` (normalized match) is shared by both.
"""

import re

_BRACE = re.compile(r"\{(.*?)\}", re.DOTALL)
# A deliberate sentinel the agent is told to print as its final line. Anchored to
# line start, case-insensitive; the grader takes the LAST occurrence so trailing
# tool output after the agent's final message cannot shadow it.
SENTINEL = "FINAL ANSWER:"
_SENTINEL_RE = re.compile(r"(?im)^[ \t>*\-]*final answer:[ \t]*(.+?)[ \t]*$")


def extract_answer(text):
    """NATIVE (single completion): last {...}, else "" (no braces → wrong)."""
    if not text:
        return ""
    m = _BRACE.findall(text)
    return m[-1].strip() if m else ""


def _strip_wrappers(s):
    """Normalize the value after the sentinel: drop markdown emphasis and one
    pair of surrounding braces/quotes the agent may add, e.g.
    'FINAL ANSWER: {11.90}', '**FINAL ANSWER:** 42' (→ '** 42'), '`11.90`'."""
    s = s.strip()
    # markdown emphasis / code ticks the model may wrap the line or value with
    s = s.strip("*`_ \t")
    for lo, hi in (("{", "}"), ('"', '"'), ("'", "'")):
        if len(s) >= 2 and s[0] == lo and s[-1] == hi:
            s = s[1:-1].strip("*`_ \t")
            break
    return s


def extract_sentinel(text):
    """AGENT (transcript): value after the LAST `FINAL ANSWER:` line, else ""."""
    if not text:
        return ""
    matches = _SENTINEL_RE.findall(text)
    return _strip_wrappers(matches[-1]) if matches else ""


def is_correct(parsed, gold):
    """Case-insensitive normalized equality (upstream: lower-cased, leading+length)."""
    return bool(parsed) and parsed.strip().lower() == (gold or "").strip().lower()


if __name__ == "__main__":
    # The agent-facing grader: score an agent TRANSCRIPT via the sentinel.
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

    parsed = extract_sentinel(text)
    reward = 1.0 if is_correct(parsed, gold) else 0.0
    with open("/logs/verifier/reward.txt", "w") as f:
        f.write(repr(reward))
    print(f"[score] gold={gold!r} parsed={parsed[:80]!r} reward={reward}")
