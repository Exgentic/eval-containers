"""Probe the stack's structured-output conformance for one walle case (model-only).

The runner container holds the case identity (EVAL_TASK_ID) and the gateway
endpoint. Each case is one JSON Schema plus an expected verdict: valid schemas
MUST be accepted (2xx), invalid schemas MUST be rejected (HTTP 400/422). We POST
the schema as `response_format.json_schema` to the gateway's OpenAI-compatible
chat/completions surface, classify the observed verdict, and write the reward.

Reward = 1.0 iff the OBSERVED verdict matches the EXPECTED verdict, else 0.0.
Infra failures (auth, wrong endpoint, 5xx, timeout, connection) are classified
as `error` (not a conformance verdict) and score 0 — a broken gateway reads as
an error, not a conformance fail.

For a valid+accepted case we additionally validate the returned content against
the schema with jsonschema and record `conforms` as a DIAGNOSTIC in the export.
It does NOT affect the reward: whether the model's output conforms is a model /
constrained-decoding-quality question; walle's contract is schema acceptance.

Fail-closed: any unexpected error leaves reward = 0.
"""

import datetime
import json
import os
import sys
import urllib.error
import urllib.request

REWARD_PATH = "/logs/verifier/reward.txt"
EXPORT_PATH = "/output/task/walle.json"
TASKS_JSONL = "/tasks/all.jsonl"

# A JSON body 400/422 from the model surface means the schema was rejected
# (unsupported/invalid `response_format`). Any other non-2xx is an infra error,
# not a verdict on the schema.
REJECT_STATUSES = {400, 422}


def write_reward(value: str) -> None:
    os.makedirs("/logs/verifier", exist_ok=True)
    with open(REWARD_PATH, "w") as f:
        f.write(value)


def write_export(data: dict) -> None:
    os.makedirs(os.path.dirname(EXPORT_PATH), exist_ok=True)
    with open(EXPORT_PATH, "w") as f:
        json.dump(data, f, indent=2)


def resolve_case(task_id: int) -> dict:
    """Map the sequential EVAL_TASK_ID to its case row.

    Prefer the per-field files the shared materializer wrote for this task; fall
    back to seeking the build-time map directly.
    """
    field_dir = f"/tasks/{task_id}"

    def read(field: str) -> str | None:
        p = f"{field_dir}/{field}.txt"
        if os.path.exists(p):
            with open(p) as f:
                return f.read()
        return None

    schema = read("schema")
    if schema is not None:
        return {
            "schema": schema.strip(),
            "expect": (read("expect") or "").strip(),
            "suite": (read("suite") or "").strip(),
            "kind": (read("kind") or "").strip(),
            "upstream_id": (read("upstream_id") or "").strip(),
        }

    with open(TASKS_JSONL) as f:
        for line in f:
            row = json.loads(line)
            if int(row["id"]) == task_id:
                return row
    raise SystemExit(f"[runner] task id {task_id} not found in {TASKS_JSONL}")


def classify(status: int) -> str:
    if 200 <= status < 300:
        return "accept"
    if status in REJECT_STATUSES:
        return "reject"
    return "error"


def probe(
    base_url: str, model: str, schema_obj: dict | str
) -> tuple[str, int, str | None, str | None, str | None]:
    """POST the schema as response_format and classify the outcome.

    Returns (verdict, http_status, content, error_message, detail) where
    `error_message` is set only for an infra `error` verdict (connection /
    timeout / 5xx), and `detail` is the stack's own response body on any
    non-2xx (the rejection reason) — a diagnostic, kept even for an expected
    `reject` so a swept report shows *why* each schema was rejected.
    """
    strict = os.environ.get("WALLE_STRICT", "false").lower() in ("1", "true", "yes")
    json_schema = {"name": "walle_case", "schema": schema_obj}
    if strict:
        json_schema["strict"] = True

    payload = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": "Return a JSON object that conforms to the provided schema.",
            }
        ],
        "response_format": {"type": "json_schema", "json_schema": json_schema},
        "max_tokens": int(os.environ.get("WALLE_MAX_TOKENS", "512")),
    }

    req = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
        method="POST",
    )
    timeout = int(os.environ.get("WALLE_TIMEOUT", "120"))
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            body = resp.read().decode("utf-8")
            status = resp.status
    except urllib.error.HTTPError as e:
        status = e.code
        body = e.read().decode("utf-8", "replace") if hasattr(e, "read") else str(e)
        verdict = classify(status)
        # `error` only for an infra error; `detail` always carries the body.
        return (
            verdict,
            status,
            None,
            (None if verdict == "reject" else body[:500]),
            body[:500],
        )
    except (urllib.error.URLError, TimeoutError, OSError) as e:
        msg = f"{type(e).__name__}: {e}"
        return "error", 0, None, msg, msg

    # 2xx — pull the model's content out for the conformance diagnostic.
    content = None
    try:
        data = json.loads(body)
        content = data["choices"][0]["message"]["content"]
    except (json.JSONDecodeError, KeyError, IndexError, TypeError):
        pass
    return classify(status), status, content, None, None


def check_conforms(content: str | None, schema_obj: dict) -> bool | None:
    """Diagnostic only: does the returned content validate against the schema?"""
    if content is None:
        return None
    try:
        import jsonschema

        instance = json.loads(content)
        jsonschema.validate(instance, schema_obj)
        return True
    except json.JSONDecodeError:
        return False
    except Exception:  # noqa: BLE001 — jsonschema.ValidationError/SchemaError etc.
        return False


def main() -> int:
    # We bypass /usr/local/bin/run, so create the output dirs write-result
    # expects and record a start time it would otherwise write.
    for d in ("/output/model", "/output/agent", "/output/task"):
        os.makedirs(d, exist_ok=True)
    if not os.path.exists("/output/agent/.started-at"):
        try:
            now = datetime.datetime.now(datetime.timezone.utc)
            with open("/output/agent/.started-at", "w") as f:
                f.write(now.strftime("%Y-%m-%dT%H:%M:%SZ"))
        except OSError:
            pass

    # Fail-closed baseline before anything can go wrong.
    write_reward("0")

    task_id = int(os.environ.get("EVAL_TASK_ID", os.environ.get("TASK_ID", "0")))
    model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or "gpt-5-mini"
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")

    case = resolve_case(task_id)
    expect = case.get("expect", "")
    suite = case.get("suite", "")
    print(f"[runner] task={task_id} suite={suite} expect={expect}", file=sys.stderr)
    print(f"[runner] model={model} base_url={base_url}", file=sys.stderr)

    export: dict = {
        "task_id": task_id,
        "suite": suite,
        "kind": case.get("kind", ""),
        "upstream_id": case.get("upstream_id", ""),
        "expect": expect,
    }

    raw_schema = case.get("schema", "")
    try:
        schema_obj = json.loads(raw_schema)
    except json.JSONDecodeError as e:
        if expect == "accept":
            # A valid case must itself be valid JSON — an unparsable one is a
            # data/store bug, not a verdict on the stack.
            export.update(verdict="error", error=f"valid case schema unparsable: {e}")
            write_export(export)
            print(f"[runner] {export['error']}", file=sys.stderr)
            return 1
        # An invalid case whose schema is not even valid JSON (upstream
        # unmarshal=false): send the raw text so the stack rejects a non-object
        # `schema` (→ 400), which is exactly the expected reject verdict.
        schema_obj = raw_schema

    verdict, status, content, error, detail = probe(base_url, model, schema_obj)
    conforms = (
        check_conforms(content, schema_obj)
        if (verdict == "accept" and expect == "accept")
        else None
    )

    reward = 1.0 if (verdict != "error" and verdict == expect) else 0.0
    export.update(
        verdict=verdict,
        http_status=status,
        conforms=conforms,  # diagnostic — not part of the reward
        error=error,
        detail=detail,  # stack's response body on any non-2xx (the reject reason)
        reward=reward,
    )
    write_export(export)
    write_reward(repr(reward))
    print(
        f"[runner] verdict={verdict} status={status} expect={expect} "
        f"conforms={conforms} reward={reward}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
