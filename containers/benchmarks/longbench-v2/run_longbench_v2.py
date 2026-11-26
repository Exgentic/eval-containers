"""LongBench v2 native harness: upstream pred.py's 0-shot setting, model-only.

One task (EVAL_TASK_ID) per run, reproducing THUDM/LongBench pred.py @ 2e00731:

  prompt    prompts/0shot.txt filled with the stripped context, question and
            choices (rendered at build time by materialize.py, byte-identical).
  truncate  query_llm: tokenize the whole prompt and, if it is longer than the
            window (config/model2maxlen.json: 120000), keep the first and the
            last max_len//2 tokens. API models use tiktoken's gpt-4o-2024-08-06
            encoding (o200k_base), with disallowed_special=() — so do we, for
            every model: the served model's own tokenizer is not in the image.
  call      one chat/completions request, the prompt as the only user message,
            temperature 0.1, max_tokens 128; up to 5 tries.
  extract   extract_answer: drop '*', the FIRST "The correct answer is (X)",
            else the first "The correct answer is X".
  judge     pred == answer.

Writes /logs/verifier/reward.txt (1 or 0) and /output/task/longbench-v2.json
(prediction, raw reply, token counts, and the task's difficulty/length for the
leaderboard's splits). Fail-closed: anything unexpected leaves reward 0.

`run_longbench_v2.py --score <reply>` grades a given reply instead of calling
the model (the oracle's path).
"""

import json
import os
import re
import sys
import time
import urllib.error
import urllib.request

REWARD_PATH = "/logs/verifier/reward.txt"
EXPORT_PATH = "/output/task/longbench-v2.json"
TRIES = 5  # pred.py query_llm: `while tries < 5`


def extract_answer(response):
    """Upstream pred.py extract_answer, verbatim."""
    response = response.replace("*", "")
    match = re.search(r"The correct answer is \(([A-D])\)", response)
    if match:
        return match.group(1)
    else:
        match = re.search(r"The correct answer is ([A-D])", response)
        if match:
            return match.group(1)
        else:
            return None


def truncate(prompt, max_len):
    """Upstream query_llm truncation for API models. Returns (prompt, n_tokens)."""
    import tiktoken

    tokenizer = tiktoken.encoding_for_model("gpt-4o-2024-08-06")
    input_ids = tokenizer.encode(prompt, disallowed_special=())
    n = len(input_ids)
    if n > max_len:
        input_ids = input_ids[: max_len // 2] + input_ids[-max_len // 2 :]
        prompt = tokenizer.decode(input_ids)
    return prompt, n


def query_llm(base_url, model, prompt, temperature, top_p, max_tokens, timeout, extra):
    """One chat/completions call, retried as upstream. Returns (text, error)."""
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
    }
    if temperature is not None:
        payload["temperature"] = temperature
    if top_p is not None:
        payload["top_p"] = top_p
    payload.update(extra)
    body = json.dumps(payload).encode("utf-8")
    err = None
    for _ in range(TRIES):
        req = urllib.request.Request(
            base_url.rstrip("/") + "/chat/completions",
            data=body,
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
            },
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
            return data["choices"][0]["message"]["content"] or "", None
        except urllib.error.HTTPError as e:
            err = f"HTTP {e.code}: {e.read().decode('utf-8', 'replace')[:300]}"
        except (OSError, KeyError, IndexError, TypeError, ValueError) as e:
            err = f"{type(e).__name__}: {e}"
        print(f"[longbench-v2] call failed, retrying: {err}", file=sys.stderr)
        time.sleep(1)
    return "", err


def read_task(task_id):
    d = f"/tasks/{task_id}"

    def read(field):
        with open(f"{d}/{field}.txt", encoding="utf-8") as f:
            return f.read()

    fields = ("id", "answer", "domain", "sub_domain", "difficulty", "length")
    return read("problem"), {k: read(k).strip() for k in fields}


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def env_float(name, default):
    """Empty string omits the parameter (the model's own default applies)."""
    v = os.environ.get(name, default)
    return float(v) if v != "" else None


def main():
    write(REWARD_PATH, "0\n")  # fail-closed baseline
    task_id = os.environ.get("EVAL_TASK_ID", "0")
    prompt, meta = read_task(task_id)
    record = {"task_id": task_id, **meta}

    if len(sys.argv) == 3 and sys.argv[1] == "--score":
        response, err = sys.argv[2], None
    else:
        model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or ""
        base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
        max_len = int(os.environ.get("LONGBENCH_V2_MAX_LEN", "120000"))
        extra = json.loads(os.environ.get("LONGBENCH_V2_EXTRA_BODY", "") or "{}")
        prompt, n_tokens = truncate(prompt, max_len)
        record.update(model=model, prompt_tokens=n_tokens, truncated=n_tokens > max_len)
        response, err = query_llm(
            base_url,
            model,
            prompt,
            env_float("LONGBENCH_V2_TEMP", "0.1"),
            env_float("LONGBENCH_V2_TOP_P", ""),
            int(os.environ.get("LONGBENCH_V2_MAX_TOKENS", "128")),
            int(os.environ.get("LONGBENCH_V2_TIMEOUT", "600")),
            extra,
        )

    response = response.strip()
    pred = extract_answer(response)
    judge = pred == meta["answer"]
    record.update(response=response[-2000:], pred=pred, judge=judge, error=err)
    write(EXPORT_PATH, json.dumps(record, indent=2) + "\n")
    write(REWARD_PATH, "1\n" if judge else "0\n")
    print(
        f"[longbench-v2] task={task_id} pred={pred} gold={meta['answer']} judge={judge}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
