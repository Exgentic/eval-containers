"""Write a task's gold solution from the call record of a run that passed it.

    solution-from-calls.py <model/calls.jsonl.zst> solutions/<upstream id>.jsonl

A solution is the actions the agent sent to the desktop, in order, each with the
wait that preceded it in the run: the model's own thinking time is what gave the
desktop time to settle. Needs `zstandard`.
"""

import json
import re
import sys

import zstandard

# One `curl … /act … -d '{"action": …}'`, or one `sleep N`, as the agent typed them.
STEP = re.compile(r"/act\b[^;&|]*?-d '([^']*)'|\bsleep ([\d.]+)")


def records(path):
    # The edge leaves the frame unfinalized: read until the stream gives out.
    with open(path, "rb") as f:
        reader = zstandard.ZstdDecompressor().stream_reader(f)
        buf = b""
        while True:
            try:
                chunk = reader.read(1 << 20)
            except zstandard.ZstdError:
                break
            if not chunk:
                break
            buf += chunk
            while b"\n" in buf:
                line, buf = buf.split(b"\n", 1)
                if line.strip():
                    yield json.loads(line)


calls = []
for r in records(sys.argv[1]):
    body = json.loads(r.get("request") or "null")
    if r.get("status") == 200 and isinstance(body, dict) and "messages" in body:
        calls.append((r["start_unix"], r["start_unix"] + r["total_ms"] / 1000, body))
calls.sort(key=lambda c: c[0])
turns = [
    m
    for m in max(calls, key=lambda c: len(c[2]["messages"]))[2]["messages"]
    if m["role"] == "assistant"
]

# Turn k is call k's answer, so its commands were issued when that call ended.
steps, last = [], calls[0][0]
for (_, issued, _), turn in zip(calls, turns):
    for block in turn["content"] if isinstance(turn["content"], list) else []:
        command = (
            (block.get("input") or {}).get("command", "")
            if block.get("type") == "tool_use"
            else ""
        )
        found, at = STEP.findall(command), issued
        if "/act" in command and not any(action for action, _ in found):
            sys.exit(
                "an action this run sent is not in its command: %s" % command[:200]
            )
        for action, slept in found:
            if action:
                steps.append(
                    {
                        "wait": round(max(at - last, 0.5), 1),
                        "action": json.loads(action)["action"],
                    }
                )
                last = at
            else:
                at += float(slept)
with open(sys.argv[2], "w") as out:
    out.writelines(json.dumps(s) + "\n" for s in steps)
print(
    "%s: %d actions over %.0fs"
    % (sys.argv[2], len(steps), sum(s["wait"] for s in steps))
)
