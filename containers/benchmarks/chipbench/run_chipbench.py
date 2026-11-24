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
           defaults), max_tokens 16384; top_p is not sent to Claude, Gemini or
           GPT-5, as sv-generate omits it there. A failed call is retried up to
           10 times, 20 s apart, as upstream, within CALL_DEADLINE seconds.
  parse  — sv-generate's non-iccad extraction: the lines between [BEGIN] and
           [DONE]; else the first ``` fenced block; else the text up to the
           first endmodule.
  grade  — grade_sv.py: iverilog compile with test.sv + ref.sv, simulate,
           sv-iv-analyze verdict. Reward 1.0 iff the verdict is pass ('.').
Fail-closed: any error leaves reward 0.0.

The grader simulates the answer, and a TopModule can end the testbench early
($finish), rewrite its counters (tb.stats1.errors) or print the verdict line
itself; upstream scores those as passes and so does this port. The answer is
scanned for those constructs and the hits are recorded as `answer_flags` in the
export, without changing the reward.
"""

import json
import os
import re
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
# Stop retrying before EVAL_TIMEOUT (1200 s) so grading and the export still run.
CALL_DEADLINE = int(os.environ.get("CHIPBENCH_CALL_DEADLINE", "1000"))
NO_TOP_P = re.compile(r"claude|gemini|gpt-5", re.I)
LOG_KEEP = 1024 * 1024  # exported log: first + last LOG_KEEP bytes
# Constructs an honest answer has no use for; none of the 223 golden modules,
# the code given in prompts, or 1,340 recorded Claude answers contains one.
ANSWER_FLAGS = {
    "testbench-reference": r"\btb\s*\.|\$root\b",
    "ends-simulation": r"\$(finish|stop)\b",
    "file-io": r"\$(fopen|fwrite|fdisplay|fclose)\b",
    "prints-verdict": r"Mismatches",
}

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


def answer_flags(sv):
    return [name for name, pat in ANSWER_FLAGS.items() if re.search(pat, sv)]


def chat(base_url, model, prompt, sampling, timeout):
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
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        body = json.loads(resp.read().decode())
    content = body["choices"][0]["message"].get("content") or ""
    if isinstance(content, list):  # sv-generate: resp.content[0]['text']
        content = content[0].get("text", "")
    return content, body.get("usage", {})


def call_model(base_url, model, real_model, prompt, export):
    # Upstream sends top_p only to the providers that accept it alongside
    # temperature: sv-generate omits it for Claude, Gemini and GPT-5. Other
    # models get it, and a 400 naming a sampling knob drops top_p, then
    # temperature — recorded in the export, never silent.
    sampling = {"temperature": TEMPERATURE}
    if not NO_TOP_P.search(real_model):
        sampling["top_p"] = TOP_P
    deadline = time.monotonic() + CALL_DEADLINE
    for attempt in range(10):
        left = deadline - time.monotonic()
        if left < 30:
            export["attempts"].append({"error": "call deadline reached"})
            break
        try:
            content, usage = chat(
                base_url, model, prompt, sampling, min(MODEL_TIMEOUT, left)
            )
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
        time.sleep(min(20, max(0, deadline - time.monotonic() - 30)))
    return None


def export_sim():
    """Copy the answer, verdict, wave.vcd (grade_sv keeps it only when small) and
    the log (first + last LOG_KEEP bytes) for trace review."""
    dst = "/output/task/verifier/sim"
    os.makedirs(dst, exist_ok=True)
    for name in ("TopModule.sv", "verdict.json", "wave.vcd"):
        if os.path.isfile(f"{WORK}/{name}"):
            shutil.copyfile(f"{WORK}/{name}", f"{dst}/{name}")
    log = f"{WORK}/sv-iv-test.log"
    if os.path.isfile(log):
        with open(log, "rb") as f, open(f"{dst}/sv-iv-test.log", "wb") as out:
            size = os.path.getsize(log)
            if size <= 2 * LOG_KEEP:
                out.write(f.read())
            else:
                out.write(f.read(LOG_KEEP))
                out.write(b"\n[... %d bytes truncated ...]\n" % (size - 2 * LOG_KEEP))
                f.seek(size - LOG_KEEP)
                out.write(f.read())


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
    real_model = os.environ.get("EVAL_MODEL") or model
    export = {
        **task,
        "model": real_model,
        "temperature": TEMPERATURE,
        "top_p": None if NO_TOP_P.search(real_model) else TOP_P,
        "max_tokens": MAX_TOKENS,
        "attempts": [],
        "reward": 0.0,
        "passfail": "no-response",
    }
    try:
        prompt = full_prompt(open(f"{tdir}/prompt.txt").read())
        resp = call_model(base_url, model, real_model, prompt, export)
        if resp is not None:
            export["response"] = resp
            os.makedirs(WORK, exist_ok=True)
            answer = os.path.join(WORK, "TopModule.sv")
            sv = extract(resp)
            with open(answer, "w") as f:
                f.write(sv)
            export["answer_flags"] = answer_flags(sv)
            res = subprocess.run(
                ["python3", "/app/grade_sv.py", tdir, answer, WORK],
                capture_output=True,
                text=True,
            )
            out = res.stdout.strip()
            reward = out if out in ("1.0", "0.0") else "0.0"
            verdict = (
                json.load(open(f"{WORK}/verdict.json"))["passfail"]
                if os.path.exists(f"{WORK}/verdict.json")
                else "grader-error"
            )
            write_reward(reward)
            export.update(reward=float(reward), passfail=verdict)
    finally:
        os.makedirs(os.path.dirname(EXPORT), exist_ok=True)
        with open(EXPORT, "w") as f:
            json.dump(export, f, indent=2)
        if os.path.isdir(WORK):
            export_sim()
    flags = export.get("answer_flags")
    print(
        f"[chipbench] {task['id']} verdict={export['passfail']} "
        f"reward={export['reward']}" + (f" answer_flags={flags}" if flags else ""),
        file=sys.stderr,
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — fail closed
        print(f"[chipbench] harness error: {e}", file=sys.stderr)
        write_reward("0.0")
