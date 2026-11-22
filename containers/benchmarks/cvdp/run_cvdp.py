"""CVDP single-turn harness for one task: prompt the model, write its files, grade.

A faithful port of upstream's non-agentic flow (NVlabs/cvdp_benchmark, pinned in
the Dockerfile):

  prompt   — src/dataset_processor.py `create_context` (non-golden branch): each
             input file in a fenced block, then "Provide me one answer for this
             request: <prompt>", then the output-file instruction. The system
             prompt is src/model_helpers.py `create_system_prompt`: the folder
             context, the per-category guidance and, for multi-file outputs, the
             JSON response schema.
  call     — src/llm_lib/openai_llm.py `prompt`: one chat/completions request
             (system + user), timeout MODEL_TIMEOUT (60 s).
  parse    — src/model_helpers.py `parse_model_response`: one output file -> the
             first fenced code block, else the whole text; several -> the
             `{"code": [{"<name>": "<code>"}]}` JSON object. An unparsable reply
             is retried up to LLM_RETRY_COUNT (3) times, as upstream.
  grade    — the input files plus the model's files are laid out under /code and
             the task's own harness runs unmodified (core/cvdp-sim/cvdp_grade.py).

Reward = 1.0 iff every harness service passes. Fail-closed: any error leaves 0.
"""

import json
import os
import re
import shutil
import subprocess
import sys
import urllib.error
import urllib.request

REWARD = "/logs/verifier/reward.txt"
EXPORT = "/output/task/cvdp.json"
MODEL_TIMEOUT = int(os.environ.get("CVDP_MODEL_TIMEOUT", "60"))
LLM_RETRY_COUNT = 3

# src/model_helpers.py ModelHelpers.__init__ (self.folders), verbatim.
FOLDERS = """You are a helpful assistance.
Consider that you have a folder structure like the following:\n
    - rtl/*   : Contains files which are RTL code.
    - verif/* : Contains files which are used to verify the correctness of the RTL code.
    - docs/*  : Contains files used to document the project, like Block Guides, RTL Plans and Verification Plans.

When generating files, return the file name in the correct place at the folder structure.
"""
SCHEMA = ['{ "code": [{ "<name>" : "<code>"}] }', '{ "response": "<response>" }']
# src/model_helpers.py create_system_prompt category_guidance — the categories
# present in the no-commercial code-generation split.
GUIDANCE = {
    2: "You are solving an 'RTL Code Completion' problem. To solve this problem correctly, you should only respond with the RTL code generated according to the requirements.",
    3: "You are solving a 'Specification to RTL Translation' problem. To solve this problem correctly, you should only respond with the RTL code translated from the specification.",
    4: "You are solving an 'RTL Code Modification' problem. To solve this problem correctly, you should only respond with the modified RTL code according to the requirements.",
    7: "You are solving an 'RTL Lint Improvement or Power-Performance Optimization' problem. To solve this problem correctly, you should only respond with improved RTL code to address lint issues or optimize for power/performance.",
    16: "You are solving an 'RTL Debugging and Bug Fixing' problem. To solve this problem correctly, you should only respond with the RTL code that is debugged and fixed to address the bug.",
}


def system_prompt(category, schema):
    s = FOLDERS + f"\n{GUIDANCE[category]}\n"
    if schema is not None:
        s += "\nProvide the response in one of the following JSON schemas: \n"
        s += "\nor\n".join(schema)
        s += "\nThe response should be in JSON format, including double-quotes around keys and values, and proper escaping of quotes within values, and escaping of newlines."
    return s


def user_prompt(task):
    p = ""
    for name, body in task["input_context"].items():
        p += f"\nConsider the following content for the file {name}:\n```\n{body}\n```"
    p += f"\nProvide me one answer for this request: {task['prompt']}\n"
    files = task["output_files"]
    if len(files) == 1:
        p += f"Please provide your response as plain text without any JSON formatting. Your response will be saved directly to: {files[0]}.\n"
    else:
        p += f"Name the files as: {files}.\n"
    return p


def code_blocks(text):
    out, pos = [], 0
    while m := re.search(r"```(?:(\w+)\n)?(.*?)```", text[pos:], re.DOTALL):
        out.append(m.group(2))
        pos += m.end()
    return out


def fix_json(content):
    """src/model_helpers.py fix_json_formatting."""
    try:
        content = re.sub(
            r"(\{|\,)\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*:", r'\1 "\2":', content
        )
        content = re.sub(
            r":\s*([a-zA-Z][a-zA-Z0-9_\s]*[a-zA-Z0-9])(\s*[,}])", r': "\1"\2', content
        )
    except Exception:  # noqa: BLE001
        pass
    return content


def parse(content, files):
    """Return {filename: code} or None (unparsable) — parse_model_response."""
    content = content.strip()
    if len(files) == 1:
        blocks = code_blocks(content)
        return {files[0]: blocks[0] if blocks else content}
    if content.startswith("{") and content.endswith("}"):
        content = fix_json(content)
    try:
        text = re.sub(r"[\x00-\x1F\x7F]", "", content)
        start, end = text.find("{"), text[::-1].find("}")
        obj = json.loads(text[start : -end if end else None])
    except Exception:  # noqa: BLE001
        return None
    result = {}
    if "code" in obj:
        for entry in obj["code"]:
            for name, code in entry.items():
                blocks = code_blocks(code) if isinstance(code, str) else []
                result[name] = blocks[0] if blocks else code
    elif "response" in obj and len(files) == 1:
        result[files[0]] = obj["response"]
    return result


def chat(base_url, model, system, user):
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
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
    return body["choices"][0]["message"].get("content") or ""


def write_reward(value):
    os.makedirs(os.path.dirname(REWARD), exist_ok=True)
    with open(REWARD, "w") as f:
        f.write(value)


def main():
    write_reward("0.0")
    tid = os.environ.get("EVAL_TASK_ID", "0")
    tdir = f"/tasks/{tid}"
    task = json.load(open(f"{tdir}/task.json"))
    model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or ""
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    files = task["output_files"]
    schema = None if len(files) == 1 else SCHEMA
    system, user = system_prompt(task["category"], schema), user_prompt(task)

    export = {
        "id": task["id"],
        "category": task["category"],
        "model": model,
        "output_files": files,
        "attempts": [],
    }
    output = None
    for attempt in range(LLM_RETRY_COUNT + 1):
        try:
            content = chat(base_url, model, system, user)
        except Exception as e:  # noqa: BLE001 — upstream retries on any call error
            export["attempts"].append({"error": str(e)})
            continue
        output = parse(content, files)
        export["attempts"].append({"parsed": output is not None, "chars": len(content)})
        if output is not None:
            break

    # Lay out /code as upstream does: the input files, then the model's files.
    shutil.rmtree("/code", ignore_errors=True)
    for name, body in {**task["input_context"], **(output or {})}.items():
        path = os.path.join("/code", name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as f:
            f.write(body if isinstance(body, str) else json.dumps(body))
    export["written"] = sorted((output or {}).keys())

    res = subprocess.run(
        ["python3", "/tests/cvdp_grade.py", tdir, "/logs/verifier/services"],
        capture_output=True,
        text=True,
    )
    reward = res.stdout.strip() or "0.0"
    write_reward(reward if reward in ("1.0", "0.0") else "0.0")
    export["reward"] = reward
    export["grade_log"] = res.stderr[-4000:]
    os.makedirs(os.path.dirname(EXPORT), exist_ok=True)
    json.dump(export, open(EXPORT, "w"), indent=2)
    os.makedirs("/output/task", exist_ok=True)
    shutil.copytree("/logs/verifier", "/output/task/verifier", dirs_exist_ok=True)
    print(f"[cvdp] {task['id']} reward={reward}", file=sys.stderr)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — fail closed
        print(f"[cvdp] harness error: {e}", file=sys.stderr)
        write_reward("0.0")
