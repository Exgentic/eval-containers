"""ZeroBench native harness — faithful, model-only reproduction (no agent).

ZeroBench's upstream protocol is single-turn vision-language inference: the model
is shown the question's image(s) + text, asked to reason and give its answer in
curly braces (`{final answer}`), and scored by extracting the last `{...}` and
matching it (lower-cased) against the gold answer. The leaderboard reports three
metrics over N=5 samples per question:

  pass@1  — mean fraction of a question's N samples that are correct
  pass@5  — the question is correct if >=1 of N samples is correct
  pass^5  — the question is correct only if ALL N samples are correct

This harness reproduces that for ONE question (EVAL_TASK_ID): it draws N samples
from the model via the gateway, scores each, and writes:
  - /logs/verifier/reward.txt  = n_correct / N   (so the mean reward over all
    questions == pass@1)
  - /output/task/zerobench.json = per-sample detail + the three per-question
    indicators, which the sweep aggregator turns into the fleet-level
    pass@1 / pass@5 / pass^5.

Fail-closed: any unexpected error leaves reward = 0.
"""

import base64
import json
import os
import sys
import urllib.error
import urllib.request

REWARD_PATH = "/logs/verifier/reward.txt"
EXPORT_PATH = "/output/task/zerobench.json"
TASKS_JSONL = "/tasks/all.jsonl"

# Upstream instruction appended to every question (zerobench README code snippet).
INSTRUCTION = (
    "\n\n\nLet's think step by step and give the final answer in curly "
    "braces, like this: {final answer}"
)
# Shared scorer — single source of truth (core/test-braced-match/score_braced.py),
# COPY'd to /app in-container; the repo path is the fallback for out-of-container
# use (e.g. a sweep script importing this module).
for _cand in (
    "/app",
    os.path.dirname(os.path.abspath(__file__)),
    os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "../../core/test-braced-match"
    ),
):
    if os.path.exists(os.path.join(_cand, "score_braced.py")):
        sys.path.insert(0, _cand)
        break
from score_braced import extract_answer, is_correct  # noqa: E402


def write_reward(value: str) -> None:
    os.makedirs("/logs/verifier", exist_ok=True)
    with open(REWARD_PATH, "w") as f:
        f.write(value)


def write_export(data: dict) -> None:
    os.makedirs(os.path.dirname(EXPORT_PATH), exist_ok=True)
    with open(EXPORT_PATH, "w") as f:
        json.dump(data, f, indent=2)


def _as_image_list(raw) -> list:
    """images_b64 is stored as a JSON-array string (so the shared str()-based
    materializer round-trips it). Accept a list too, for robustness."""
    if isinstance(raw, list):
        return raw
    if isinstance(raw, str) and raw.strip():
        try:
            v = json.loads(raw)
            return v if isinstance(v, list) else []
        except json.JSONDecodeError:
            return []
    return []


def resolve_task(task_id: int) -> dict:
    """Prefer the per-field files the shared materializer wrote; else seek the map."""
    field_dir = f"/tasks/{task_id}"

    def read(field: str) -> str | None:
        p = f"{field_dir}/{field}.txt"
        if os.path.exists(p):
            with open(p) as f:
                return f.read()
        return None

    problem = read("problem")
    if problem is not None:
        return {
            "problem": problem,
            "answer": (read("answer") or "").strip(),
            "images_b64": _as_image_list(read("images_b64")),
            "id": (read("id") or str(task_id)).strip(),
        }

    with open(TASKS_JSONL) as f:
        for lineno, line in enumerate(f):
            if lineno == task_id:
                row = json.loads(line)
                row["images_b64"] = _as_image_list(row.get("images_b64"))
                return row
    raise SystemExit(f"[zerobench] task id {task_id} not found in {TASKS_JSONL}")


def sniff_mime(raw: bytes) -> str:
    if raw[:3] == b"\xff\xd8\xff":
        return "image/jpeg"
    if raw[:8] == b"\x89PNG\r\n\x1a\n":
        return "image/png"
    if raw[:4] == b"RIFF" and raw[8:12] == b"WEBP":
        return "image/webp"
    if raw[:6] in (b"GIF87a", b"GIF89a"):
        return "image/gif"
    return "image/jpeg"  # ZeroBench images are predominantly JPEG


def build_content(problem: str, images_b64: list) -> list:
    content: list = [{"type": "text", "text": problem.strip() + INSTRUCTION}]
    for b64 in images_b64:
        try:
            mime = sniff_mime(base64.b64decode(b64)[:16])
        except Exception:  # noqa: BLE001
            mime = "image/jpeg"
        content.append(
            {
                "type": "image_url",
                "image_url": {"url": f"data:{mime};base64,{b64}"},
            }
        )
    return content


def sample_once(
    base_url: str,
    model: str,
    content: list,
    temperature,
    top_p,
    max_tokens: int,
    timeout: int,
    extra_body: dict | None = None,
) -> tuple[str | None, str | None]:
    """One chat/completions call. Returns (content_text, error_message).

    temperature / top_p are omitted when None (reasoning models often require
    defaults, e.g. temperature=1 with extended thinking). extra_body is merged
    into the payload for provider-specific knobs (e.g. a thinking/effort config)
    — this keeps the harness model-agnostic while allowing a leaderboard-matching
    run to enable the model's thinking.
    """
    payload: dict = {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": max_tokens,
    }
    if temperature is not None:
        payload["temperature"] = temperature
    if top_p is not None:
        payload["top_p"] = top_p
    if extra_body:
        payload.update(extra_body)
    req = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        return data["choices"][0]["message"]["content"], None
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", "replace") if hasattr(e, "read") else str(e)
        return None, f"HTTP {e.code}: {body[:300]}"
    except (
        urllib.error.URLError,
        TimeoutError,
        OSError,
        KeyError,
        IndexError,
        json.JSONDecodeError,
    ) as e:
        return None, f"{type(e).__name__}: {e}"


def main() -> int:
    write_reward("0")  # fail-closed baseline

    task_id = int(os.environ.get("EVAL_TASK_ID", os.environ.get("TASK_ID", "0")))
    model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or "gpt-5-mini"
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")

    n = int(os.environ.get("ZEROBENCH_N", "5"))
    # Empty string => omit the parameter (let the model use its default — required
    # for reasoning models, where thinking forces temperature=1).
    _t = os.environ.get("ZEROBENCH_TEMP", "0.7")
    _p = os.environ.get("ZEROBENCH_TOP_P", "0.95")
    temperature = float(_t) if _t != "" else None
    top_p = float(_p) if _p != "" else None
    max_tokens = int(os.environ.get("ZEROBENCH_MAX_TOKENS", "8192"))
    timeout = int(os.environ.get("ZEROBENCH_TIMEOUT", "300"))
    # Provider-specific knobs merged into the request (e.g. enabling thinking):
    #   ZEROBENCH_EXTRA_BODY='{"thinking":{"type":"adaptive"},"output_config":{"effort":"high"}}'
    extra_body = None
    _eb = os.environ.get("ZEROBENCH_EXTRA_BODY", "")
    if _eb.strip():
        try:
            extra_body = json.loads(_eb)
        except json.JSONDecodeError as e:
            print(f"[zerobench] bad ZEROBENCH_EXTRA_BODY: {e}", file=sys.stderr)

    task = resolve_task(task_id)
    gold = str(task.get("answer", "")).strip()
    images = task.get("images_b64", []) or []
    content = build_content(str(task.get("problem", "")), images)

    print(
        f"[zerobench] task={task_id} id={task.get('id')} model={model} "
        f"n_images={len(images)} N={n}",
        file=sys.stderr,
    )

    samples = []
    n_correct = 0
    for i in range(n):
        text, err = sample_once(
            base_url,
            model,
            content,
            temperature,
            top_p,
            max_tokens,
            timeout,
            extra_body,
        )
        parsed = extract_answer(text)
        correct = bool(parsed) and is_correct(parsed, gold)
        n_correct += int(correct)
        samples.append(
            {
                "i": i,
                "extracted": parsed,
                "correct": correct,
                "error": err,
                "raw": (text or "")[-1000:],  # tail only; keep export small
            }
        )
        print(
            f"[zerobench]   sample {i}: extracted={parsed!r} correct={correct} "
            f"err={err}",
            file=sys.stderr,
        )

    pass_at_1 = n_correct / n if n else 0.0  # per-question mean
    pass_at_5 = 1.0 if n_correct >= 1 else 0.0  # >=1 correct
    pass_hat_5 = 1.0 if n_correct == n else 0.0  # all correct

    write_export(
        {
            "task_id": task_id,
            "upstream_id": task.get("id"),
            "model": model,
            "n_samples": n,
            "n_correct": n_correct,
            "gold": gold,
            "pass_at_1": pass_at_1,
            "pass_at_5": pass_at_5,
            "pass_hat_5": pass_hat_5,
            "samples": samples,
        }
    )
    # Reward = per-question pass@1 so the swept mean reward == fleet pass@1.
    write_reward(repr(pass_at_1))
    print(
        f"[zerobench] n_correct={n_correct}/{n} pass@1={pass_at_1} "
        f"pass@5={pass_at_5} pass^5={pass_hat_5}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
