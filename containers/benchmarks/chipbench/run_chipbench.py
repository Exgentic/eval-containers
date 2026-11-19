"""ChipBench native harness for one task: prompt the model once, extract the code, grade.

A faithful port of upstream's flow (zhongkaiyu/ChipBench, pinned in the Dockerfile),
which is VerilogEval v2's `make` harness:
  prompt — scripts/sv-generate, run as the Makefile runs it with configure's
           defaults (--task=spec-to-rtl, --examples=0, no --rules, no --explain):
           the spec-to-rtl system message, then "Question:" + the problem's
           prompt.txt + the [BEGIN]/[DONE] instruction + "Answer:". The same
           path serves generation and debugging (debug prompts carry the buggy
           code, and in one-shot also the .vcd waveform, inline).
  call   — one chat completion, temperature 0.85, top_p 0.95 (configure
           defaults), max_tokens 16384; a failed call is retried up to 10 times,
           20 s apart, as upstream.
  parse  — sv-generate's non-iccad extraction: the lines between [BEGIN] and
           [DONE]; else the first ``` fenced block; else the text up to the
           first endmodule.
  grade  — grade_sv.py: iverilog compile with test.sv + ref.sv, simulate,
           sv-iv-analyze verdict. Reward 1.0 iff the verdict is pass ('.').
Fail-closed: any error leaves reward 0.0.
"""

import json
import os
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request

REWARD = "/logs/verifier/reward.txt"
EXPORT = "/output/task/chipbench.json"
WORK = "/logs/verifier/sim"
TEMPERATURE, TOP_P, MAX_TOKENS = 0.85, 0.95, 16384
MODEL_TIMEOUT = int(os.environ.get("CHIPBENCH_MODEL_TIMEOUT", "600"))

# scripts/sv-generate, verbatim.
SYSTEM_MSG = """
You are a Verilog RTL designer that only writes code using correct Verilog syntax.
"""
NO_EXPLAIN_SUFFIX = """
Enclose your code with [BEGIN] and [DONE]. Only output the code snippet
and do NOT output anything else.
"""


def full_prompt(prompt):
    p = "\nQuestion:\n" + prompt.strip() + "\n"
    p = p.rstrip() + "\n" + NO_EXPLAIN_SUFFIX
    return p + "\nAnswer:\n"


def extract(resp):
    """sv-generate's non-iccad branch: returns the .sv file text it writes."""
    lines = resp.splitlines()
    out = [""]
    backticks = sum(1 for line in lines if line.startswith("```"))
    module_exists = endmodule_first = False
    for line in lines:
        if line.startswith("```"):
            continue
        if line.startswith("endmodule"):
            if not module_exists:
                endmodule_first = True
        elif line.startswith("module TopModule"):
            module_exists = True
    if endmodule_first:
        module_exists = False

    code, start, end = [], False, False
    for line in lines:
        if not start:
            if line.strip() == "[BEGIN]":
                start = True
            elif line.lstrip().startswith("[BEGIN]"):
                code.append(line.lstrip().replace("[BEGIN]", ""))
                start = True
        elif not end:
            if line.strip() == "[DONE]":
                end = True
            elif line.rstrip().endswith("[DONE]"):
                code.append(line.rstrip().replace("[DONE]", ""))
                end = True
            else:
                code.append(line)
    if start and end:
        out += code
    if not start and not end:
        first = second = found_module = found_end = False
        for line in lines:
            echo = True
            if line.strip().startswith("module TopModule"):
                found_module = True
            if backticks >= 2:
                if (not first) or second:
                    echo = False
            else:
                if found_end:
                    echo = False
                if module_exists and not found_module:
                    echo = False
            if line.startswith("```"):
                if not first:
                    first = True
                else:
                    second = True
                echo = False
            elif line.strip().startswith("endmodule"):
                found_end = True
            if echo:
                out.append(line)
        out += ["", "// VERILOG-EVAL: response did not use <CODE></CODE> correctly"]
    return "\n".join(out + [""]) + "\n"


def chat(base_url, model, prompt, sampling):
    payload = {
        "model": model,
        "max_tokens": MAX_TOKENS,
        **sampling,
        "messages": [
            {"role": "system", "content": SYSTEM_MSG},
            {"role": "user", "content": prompt},
        ],
    }
    req = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=json.dumps(payload).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=MODEL_TIMEOUT) as resp:
        body = json.loads(resp.read().decode())
    content = body["choices"][0]["message"].get("content") or ""
    if isinstance(content, list):  # sv-generate: resp.content[0]['text']
        content = content[0].get("text", "")
    return content, body.get("usage", {})


def call_model(base_url, model, prompt, export):
    # Upstream sends top_p only to the providers that accept it alongside
    # temperature (sv-generate omits it for Claude/Gemini/GPT-5). The gateway
    # hides the provider, so a 400 naming a sampling knob drops top_p, then
    # temperature — recorded in the export, never silent.
    sampling = {"temperature": TEMPERATURE, "top_p": TOP_P}
    for attempt in range(10):
        try:
            content, usage = chat(base_url, model, prompt, sampling)
            export["attempts"].append(
                {"ok": True, "sampling": dict(sampling), "usage": usage}
            )
            return content
        except urllib.error.HTTPError as e:
            detail = e.read().decode(errors="replace")[:500]
            export["attempts"].append(
                {
                    "error": f"HTTP {e.code}",
                    "detail": detail,
                    "sampling": dict(sampling),
                }
            )
            if (
                e.code == 400
                and sampling
                and any(k in detail for k in ("top_p", "temperature"))
            ):
                sampling.pop("top_p" if "top_p" in sampling else "temperature")
                continue
        except Exception as e:  # noqa: BLE001 — upstream retries on any error
            export["attempts"].append(
                {"error": str(e)[:500], "sampling": dict(sampling)}
            )
        time.sleep(20)
    return None


def write_reward(value):
    os.makedirs(os.path.dirname(REWARD), exist_ok=True)
    with open(REWARD, "w") as f:
        f.write(value)


def main():
    write_reward("0.0")
    tdir = f"/tasks/{os.environ.get('EVAL_TASK_ID', '0')}"
    task = json.load(open(f"{tdir}/task.json"))
    model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or ""
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    # MODEL is the gateway's alias (eval-model); EVAL_MODEL names the real one.
    export = {
        **task,
        "model": os.environ.get("EVAL_MODEL") or model,
        "temperature": TEMPERATURE,
        "top_p": TOP_P,
        "max_tokens": MAX_TOKENS,
        "attempts": [],
    }

    resp = call_model(
        base_url, model, full_prompt(open(f"{tdir}/prompt.txt").read()), export
    )
    reward, verdict = "0.0", "no-response"
    if resp is not None:
        os.makedirs(WORK, exist_ok=True)
        answer = os.path.join(WORK, "TopModule.sv")
        with open(answer, "w") as f:
            f.write(extract(resp))
        res = subprocess.run(
            ["python3", "/app/grade_sv.py", tdir, answer, WORK],
            capture_output=True,
            text=True,
        )
        reward = res.stdout.strip() if res.stdout.strip() in ("1.0", "0.0") else "0.0"
        verdict = (
            json.load(open(f"{WORK}/verdict.json"))["passfail"]
            if os.path.exists(f"{WORK}/verdict.json")
            else "grader-error"
        )
        export["response"] = resp
    write_reward(reward)
    export.update(reward=float(reward), passfail=verdict)
    os.makedirs(os.path.dirname(EXPORT), exist_ok=True)
    json.dump(export, open(EXPORT, "w"), indent=2)
    # The extracted answer, simulator log and verdict, for trace review.
    if os.path.isdir(WORK):
        shutil.copytree(WORK, "/output/task/verifier/sim", dirs_exist_ok=True)
    print(
        f"[chipbench] {task['id']} verdict={verdict} reward={reward}", file=sys.stderr
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — fail closed
        print(f"[chipbench] harness error: {e}", file=sys.stderr)
        write_reward("0.0")
