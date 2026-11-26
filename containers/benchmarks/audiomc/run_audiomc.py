"""Audio MultiChallenge native harness — the model's final turn, from audio.

AudioMC's protocol is fixed-context: the whole conversation up to the final
user turn is pre-seeded and only the model's reply to that last turn is graded.
"User turns are provided in context as base64-encoded audio files, while
assistant turns are provided as raw text" (arXiv 2512.14865, Sec. 4) — so the
user transcripts are never sent; the model has to listen. No system prompt.

This harness resolves EVAL_TASK_ID to /tasks/<id>/, sends that conversation to
the gateway's OpenAI-compatible chat surface as `input_audio` (wav) user parts
and text assistant messages, and prints the reply to stdout, which the launcher
keeps as /output/agent/stdout.log for /grade.sh to judge. Nothing is printed
when no reply comes back, so the grader scores the task 0.

Knobs (runtime env):
  AUDIOMC_TIMEOUT     per-request seconds (default EVAL_TIMEOUT - 60)
  AUDIOMC_EXTRA_BODY  JSON object merged into the request, e.g. a thinking
                      config — the paper reports "(Thinking)" variants
"""

import base64
import json
import os
import sys
import time
import urllib.error
import urllib.request


def build_messages(task_dir: str) -> list:
    with open(f"{task_dir}/turns.json") as f:
        turns = json.load(f)
    messages = []
    for turn in turns:
        with open(f"{task_dir}/{turn['audio']}", "rb") as f:
            audio = base64.b64encode(f.read()).decode("ascii")
        messages.append(
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_audio",
                        "input_audio": {"data": audio, "format": "wav"},
                    }
                ],
            }
        )
        # Upstream's history builder skips an empty assistant transcript; so
        # does this (one conversation has one).
        if turn.get("assistant"):
            messages.append({"role": "assistant", "content": turn["assistant"]})
    return messages


def main() -> int:
    task_id = os.environ.get("EVAL_TASK_ID", os.environ.get("TASK_ID", "0"))
    model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or "default"
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    timeout = int(
        os.environ.get("AUDIOMC_TIMEOUT")
        or max(60, int(os.environ.get("TIMEOUT", "600")) - 60)
    )

    payload = {"model": model, "messages": build_messages(f"/tasks/{task_id}")}
    extra = os.environ.get("AUDIOMC_EXTRA_BODY", "").strip()
    if extra:
        payload.update(json.loads(extra))
    body = json.dumps(payload).encode("utf-8")
    print(
        f"[audiomc] task={task_id} model={model} "
        f"turns={sum(m['role'] == 'user' for m in payload['messages'])} "
        f"request_bytes={len(body)}",
        file=sys.stderr,
    )

    req = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
        method="POST",
    )
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
            text = data["choices"][0]["message"]["content"] or ""
            if text.strip():
                print(text)
                return 0
            print(f"[audiomc] empty reply: {json.dumps(data)[:500]}", file=sys.stderr)
        except urllib.error.HTTPError as e:
            print(
                f"[audiomc] HTTP {e.code}: {e.read().decode('utf-8', 'replace')[:500]}",
                file=sys.stderr,
            )
            if e.code < 500 and e.code != 429:
                return 1
        except (OSError, KeyError, IndexError, json.JSONDecodeError) as e:
            print(f"[audiomc] {type(e).__name__}: {e}", file=sys.stderr)
        time.sleep(5 * (attempt + 1))
    return 1


if __name__ == "__main__":
    sys.exit(main())
