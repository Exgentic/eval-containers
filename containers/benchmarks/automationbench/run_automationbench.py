"""Run one AutomationBench task through its native harness (model-only).

The runner container holds the task identity (EVAL_TASK_ID) and the gateway
endpoint. AutomationBench addresses tasks by NAME, so we resolve the sequential
id to its task_name via the build-time map (/tasks/all.jsonl or the
materialized /tasks/$EVAL_TASK_ID/task_name.txt), then invoke the upstream
`auto-bench` CLI pointed at the gateway. The CLI runs its built-in tool-calling
loop over the simulated WorldState and grades in-process; we read the exported
result and write the reward.

Reward = task_completed_correctly (0/1): the exported per-task `passed` is
`reward == 1.0`, and with the rubric weights (partial_credit=1.0,
task_completed_correctly=0.0) `reward == partial_credit`, so
`passed == (partial_credit == 1.0) == task_completed_correctly`.

Fail-closed: any error leaves reward = 0.
"""

import json
import os
import subprocess
import sys

REWARD_PATH = "/logs/verifier/reward.txt"
EXPORT_PATH = "/output/task/automationbench.json"
TASKS_JSONL = "/tasks/all.jsonl"


def write_reward(value: str) -> None:
    os.makedirs("/logs/verifier", exist_ok=True)
    with open(REWARD_PATH, "w") as f:
        f.write(value)


def resolve_task_name(task_id: int) -> str:
    """Map the sequential EVAL_TASK_ID to its upstream task_name.

    Prefer the field the shared materializer wrote for this task; fall back to
    seeking the build-time map directly.
    """
    materialized = f"/tasks/{task_id}/task_name.txt"
    if os.path.exists(materialized):
        with open(materialized) as f:
            name = f.read().strip()
        if name:
            return name

    with open(TASKS_JSONL) as f:
        for line in f:
            row = json.loads(line)
            if int(row["id"]) == task_id:
                return row["task_name"]
    raise SystemExit(f"[runner] task id {task_id} not found in {TASKS_JSONL}")


def main() -> int:
    # We bypass /usr/local/bin/run, so create the output dirs write-result
    # expects and record a start time it would otherwise write.
    for d in ("/output/model", "/output/agent", "/output/task"):
        os.makedirs(d, exist_ok=True)
    if not os.path.exists("/output/agent/.started-at"):
        try:
            import datetime

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

    task_name = resolve_task_name(task_id)
    print(f"[runner] task={task_id} name={task_name}", file=sys.stderr)
    print(f"[runner] model={model} base_url={base_url}", file=sys.stderr)

    os.makedirs(os.path.dirname(EXPORT_PATH), exist_ok=True)

    env = dict(os.environ)
    # The key is a placeholder; the real credential lives on the gateway.
    env.setdefault("OPENAI_API_KEY", "sk-proxy")
    # Keep HF's dataset cache on a writable path (datasets is pulled in by the
    # verifiers env even though tasks are in-package Python).
    env.setdefault("HF_HOME", "/tmp/hf")
    os.makedirs(env["HF_HOME"], exist_ok=True)

    cmd = [
        "auto-bench",
        "--tasks",
        task_name,
        # Force the OpenAI-compatible path — otherwise claude-*/gemini-* model
        # names auto-route to native provider APIs instead of the gateway.
        "--api",
        "chat_completions",
        "--base-url",
        base_url,
        "--api-key-var",
        "OPENAI_API_KEY",
        "--model",
        model,
        "--domains",
        "public",
        "--max-steps",
        os.environ.get("AUTOMATIONBENCH_MAX_STEPS", "50"),
        "--max-concurrent",
        "1",
        "--export-json",
        EXPORT_PATH,
    ]
    print(f"[runner] {' '.join(cmd)}", file=sys.stderr)

    proc = subprocess.run(cmd, env=env)
    if proc.returncode != 0:
        print(f"[runner] auto-bench exited {proc.returncode}", file=sys.stderr)

    # Read the exported result and translate `passed` → reward (0/1).
    try:
        with open(EXPORT_PATH) as f:
            data = json.load(f)
        tasks = data.get("tasks") or []
        if not tasks:
            print("[runner] export has no tasks; reward stays 0", file=sys.stderr)
            return 1
        passed = bool(tasks[0].get("passed", False))
        write_reward("1" if passed else "0")
        print(f"[runner] task_completed_correctly={int(passed)}", file=sys.stderr)
        return 0
    except (OSError, json.JSONDecodeError, KeyError, IndexError) as e:
        print(f"[runner] could not read export {EXPORT_PATH}: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
