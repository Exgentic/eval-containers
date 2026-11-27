"""MMMU-Pro native harness: model-only, upstream protocol (no agent).

Reproduces MMMU-Benchmark/MMMU `mmmu-pro/` at b26bdefc7e5e007962fde8f93b2bb53c99c8d13d
for ONE question (EVAL_TASK_ID): one chat/completions call through the gateway,
graded by upstream's answer parser. Shared by `mmmu-pro` (standard, 10 or 4
options) and `mmmu-pro-vision` (the question is in a screenshot).

  prompt  — infer/infer_gpt.py:43-56 construct_prompt + prompts.yaml (standard),
            or prompts.yaml alone (vision, process_prompt:166-168)
  images  — infer_gpt.py:38-62: <image N> becomes <image>, and image_N is sent
            once per occurrence, in the order the tokens appear, after the text
  request — infer_gpt.py:116-138, 170: one user message, max_tokens 4096, no
            temperature/top_p (the API's defaults)
  score   — evaluate.py:16-33, 215-290, 429-443: parse_multi_choice_response, then
            equality with the gold letter

Writes /logs/verifier/reward.txt (1.0 / 0.0) and /output/task/mmmu_pro.json.
Fail-closed: any unexpected error leaves reward = 0.
"""

import ast
import base64
import http.client
import json
import os
import random
import re
import sys
import time
import urllib.error
import urllib.request

REWARD_PATH = "/logs/verifier/reward.txt"
EXPORT_PATH = "/output/task/mmmu_pro.json"

# prompts.yaml, verbatim.
PROMPTS = {
    "cot": {
        "vision": "Write out the multiple-choice question in the image and then solve it. The last line of your response should be of the following format: 'Answer: $LETTER' (without quotes) where LETTER is one of options. Think step by step before answering.",
        "standard": "Answer the preceding multiple choice question. The last line of your response should be of the following format: 'Answer: $LETTER' (without quotes) where LETTER is one of options. Think step by step before answering.",
    },
    "direct": {
        "vision": "Answer with the option letter from the given choices directly. The last line of your response should be of the following format: 'Answer: $LETTER' (without quotes) where LETTER is one of options.",
        "standard": "Answer with the option letter from the given choices directly.",
    },
}
SETTINGS = ("standard (10 options)", "standard (4 options)", "vision")


# ── infer/infer_gpt.py ───────────────────────────────────────────────────────


def replace_images_tokens(input_string):
    image_order = [int(num) for num in re.findall(r"<image\s+(\d+)>", input_string)]
    input_string = re.sub(r"<image\s+\d+>", "<image>", input_string)
    return input_string, image_order


def parse_options(options):
    option_letters = [chr(ord("A") + i) for i in range(len(options))]
    return "\n".join(
        f"{option_letter}. {option}"
        for option_letter, option in zip(option_letters, options)
    )


def construct_prompt(question, options, mode):
    return f"{question}\n{parse_options(options)}\n{PROMPTS[mode]['standard']}"


# ── evaluate.py ──────────────────────────────────────────────────────────────


def get_multi_choice_info(options):
    all_choices = [chr(ord("A") + i) for i in range(len(options))]
    return dict(zip(all_choices, options)), all_choices


def parse_multi_choice_response(response, all_choices, index2ans, rng):
    """Upstream's parser, unchanged except that the last-resort random guess
    draws from `rng` (seeded per question) rather than the module-level
    `random`, whose draw depends on every earlier question in the file.
    Returns (letter, guessed)."""
    last_answer_pos = response.rfind("Answer:")
    if last_answer_pos != -1:
        answer_str = response[last_answer_pos + len("Answer:") :].strip()
        matching_options = [option for option in all_choices if option in answer_str]
        if len(matching_options) == 1:
            return matching_options[0], False

    for char in [",", ".", "!", "?", ";", ":", "'"]:
        response = response.strip(char)
    response = " " + response + " "  # add space to avoid partial match

    index_ans = True
    ans_with_brack = False
    candidates = []
    for choice in all_choices:  # e.g., (A) (B) (C) (D)
        if f"({choice})" in response:
            candidates.append(choice)
            ans_with_brack = True

    if len(candidates) == 0:
        for choice in all_choices:  # e.g., A B C D
            if f"{choice} " in response:
                candidates.append(choice)

    if len(candidates) == 0:
        for choice in all_choices:  # e.g., A. B. C. D.
            if f"{choice}." in response:
                candidates.append(choice)

    if len(candidates) == 0 and len(response.split()) > 5:
        for index, ans in index2ans.items():
            if ans.lower() in response.lower():
                candidates.append(index)
                index_ans = False  # it's content ans.

    if len(candidates) == 0:  # still not get answer, randomly choose one.
        return rng.choice(all_choices), True
    if len(candidates) > 1:
        start_indexes = []
        if index_ans:
            if ans_with_brack:
                for can in candidates:
                    start_indexes.append(response.rfind(f"({can})"))
            else:
                for can in candidates:
                    start_indexes.append(response.rfind(f" {can} "))
        else:
            for can in candidates:
                start_indexes.append(response.lower().rfind(index2ans[can].lower()))
        # get the last one (np.argmax: the first index of the maximum)
        return candidates[start_indexes.index(max(start_indexes))], False
    return candidates[0], False


# ── harness ──────────────────────────────────────────────────────────────────


def write_reward(value: str) -> None:
    os.makedirs(os.path.dirname(REWARD_PATH), exist_ok=True)
    with open(REWARD_PATH, "w") as f:
        f.write(value)


def read_field(task_id: int, field: str) -> str | None:
    """A field the shared materializer wrote to /tasks/<id>/<field>.txt."""
    path = f"/tasks/{task_id}/{field}.txt"
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return f.read()


def sniff_mime(raw: bytes) -> str:
    """The stored bytes' real type: the dataset holds a few JPEGs among its
    PNGs (the loader admits only those two), and a provider may reject a data
    URL whose type does not match."""
    return "image/jpeg" if raw[:3] == b"\xff\xd8\xff" else "image/png"


def image_part(path: str) -> dict:
    with open(path, "rb") as f:
        raw = f.read()
    b64 = base64.b64encode(raw).decode("ascii")
    return {
        "type": "image_url",
        "image_url": {"url": f"data:{sniff_mime(raw)};base64,{b64}"},
    }


def build_request(task_id: int, setting: str, mode: str):
    """-> (content parts, options, gold) for one question in one setting."""
    if setting == "vision":
        if not os.path.exists(f"/tasks/images/{task_id}.png"):
            raise ValueError("this image has no 'vision' data")
        options = ast.literal_eval(read_field(task_id, "options"))
        gold = read_field(task_id, "answer").strip()
        prompt = PROMPTS[mode]["vision"]
        images = [f"/tasks/images/{task_id}.png"]
    else:
        suffix = "" if setting == "standard (10 options)" else "_4"
        if (
            read_field(task_id, "question") is None
            or read_field(task_id, "options" + suffix) is None
        ):
            raise ValueError(f"this image has no {setting!r} data")
        options = ast.literal_eval(read_field(task_id, "options" + suffix))
        gold = read_field(task_id, "answer" + suffix).strip()
        prompt, order = replace_images_tokens(
            construct_prompt(read_field(task_id, "question"), options, mode)
        )
        images = [f"/tasks/images/{task_id}_{n}.png" for n in order]
    content = [{"type": "text", "text": prompt}] + [image_part(p) for p in images]
    return content, options, gold


def request_once(
    base_url, model, content, temperature, top_p, max_tokens, timeout, extra_body
):
    """One chat/completions call, retrying a 429/5xx up to 3 times with backoff.
    Returns (content_text, finish_reason, error_message)."""
    payload: dict = {"model": model, "messages": [{"role": "user", "content": content}]}
    if max_tokens is not None:
        payload["max_tokens"] = max_tokens
    if temperature is not None:
        payload["temperature"] = temperature
    if top_p is not None:
        payload["top_p"] = top_p
    if extra_body:
        payload.update(extra_body)
    body = json.dumps(payload).encode("utf-8")
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
    }
    for attempt in range(4):
        req = urllib.request.Request(
            base_url.rstrip("/") + "/chat/completions",
            data=body,
            headers=headers,
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
            choice = data["choices"][0]
            return choice["message"]["content"] or "", choice.get("finish_reason"), None
        except urllib.error.HTTPError as e:
            err = f"HTTP {e.code}: {e.read().decode('utf-8', 'replace')[:300]}"
            if (e.code == 429 or e.code >= 500) and attempt < 3:
                time.sleep(5 * 2**attempt)
                continue
            return None, None, err
        except (
            urllib.error.URLError,
            http.client.HTTPException,
            OSError,
            KeyError,
            IndexError,
            TypeError,
            UnicodeDecodeError,
            json.JSONDecodeError,
        ) as e:
            return None, None, f"{type(e).__name__}: {e}"


def knobs() -> dict:
    """The EVAL_MMMU_PRO_* knobs, validated. Unset or empty takes upstream's
    value; an empty TEMP/TOP_P omits the parameter (upstream sets neither), and
    MAX_TOKENS=none omits max_tokens, for a model that takes
    max_completion_tokens through EXTRA_BODY instead."""
    env = lambda k: os.environ.get(f"EVAL_MMMU_PRO_{k}") or ""  # noqa: E731
    k = {
        "setting": env("SETTING") or "standard (10 options)",
        "prompt": env("PROMPT") or "cot",
        "temperature": float(env("TEMP")) if env("TEMP") else None,
        "top_p": float(env("TOP_P")) if env("TOP_P") else None,
        "max_tokens": None
        if env("MAX_TOKENS") == "none"
        else int(env("MAX_TOKENS") or 4096),
        "timeout": int(env("TIMEOUT") or 300),
        "extra_body": json.loads(env("EXTRA_BODY"))
        if env("EXTRA_BODY").strip()
        else None,
    }
    if k["setting"] not in SETTINGS or k["prompt"] not in PROMPTS:
        raise ValueError(f"unknown setting {k['setting']!r} or prompt {k['prompt']!r}")
    eb = k["extra_body"]
    if eb is not None and (
        not isinstance(eb, dict) or {"model", "messages"} & eb.keys()
    ):
        raise ValueError(
            "EXTRA_BODY must be a JSON object that sets neither model nor messages"
        )
    return k


def run(task_id: int, export: dict) -> int:
    model = os.environ.get("MODEL") or os.environ["EVAL_MODEL"]
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    k = knobs()
    export.update(model=model, **k)
    content, options, gold = build_request(task_id, k["setting"], k["prompt"])
    export["gold"] = gold
    print(
        f"[mmmu-pro] task={task_id} id={export['upstream_id']} setting={k['setting']!r} "
        f"prompt={k['prompt']} model={model} n_images={len(content) - 1}",
        file=sys.stderr,
    )

    text, finish, err = request_once(
        base_url,
        model,
        content,
        k["temperature"],
        k["top_p"],
        k["max_tokens"],
        k["timeout"],
        k["extra_body"],
    )
    parsed, guessed = None, False
    if err is None:
        index2ans, all_choices = get_multi_choice_info(options)
        rng = random.Random(f"42:{export['upstream_id']}")  # upstream seeds 42
        parsed, guessed = parse_multi_choice_response(text, all_choices, index2ans, rng)
    export.update(
        parsed=parsed,
        guessed=guessed,
        correct=parsed == gold,
        finish_reason=finish,
        error=err,
        response_tail=(text or "")[-2000:],
    )
    write_reward("1.0" if parsed == gold else "0.0")
    print(
        f"[mmmu-pro] gold={gold} parsed={parsed} guessed={guessed} "
        f"finish={finish} err={err}",
        file=sys.stderr,
    )
    # A failed request is an errored attempt, not an answer: exit non-zero so the
    # launcher records the error (output rule 17); the reward stays 0.
    return 1 if err else 0


def main() -> int:
    write_reward("0")  # fail-closed baseline
    task_id = int(os.environ.get("EVAL_TASK_ID", os.environ.get("TASK_ID", "0")))
    export = {
        "task_id": task_id,
        "upstream_id": (read_field(task_id, "id") or "").strip(),
        "subject": (read_field(task_id, "subject") or "").strip(),
        "correct": False,
        "error": None,
    }
    try:
        rc = run(task_id, export)
    except (Exception, SystemExit) as e:  # a bad knob or task: fail closed, recorded
        export["error"] = f"{type(e).__name__}: {e}"
        print(f"[mmmu-pro] {export['error']}", file=sys.stderr)
        rc = 1
    os.makedirs(os.path.dirname(EXPORT_PATH), exist_ok=True)
    with open(EXPORT_PATH, "w") as f:
        json.dump(export, f, indent=2)
    return rc


if __name__ == "__main__":
    sys.exit(main())
