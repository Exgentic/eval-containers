"""Run one BFCL task through its native harness + AST checker (model-only).

The runner container holds the task identity (EVAL_TASK_ID) and the gateway
endpoint. BFCL addresses tasks by its own string id within a category, so we
resolve the sequential id to (category, bfcl_id) via the build-time map
(/tasks/all.jsonl or the materialized /tasks/$EVAL_TASK_ID/*.txt), then drive
BFCL in-process:

  1. Register a ModelConfig for the gateway handle so BFCL's OpenAI-compatible
     handler (native function-calling mode) points at the fleet gateway. BFCL
     refuses any --model not in MODEL_CONFIG_MAPPING, and the gateway handle is
     not a known model — so we inject one at runtime rather than editing the
     installed package.
  2. Write test_case_ids_to_generate.json selecting the single task and call
     the generation entrypoint in --run-ids mode (one model turn, tool_calls).
  3. Call the evaluation entrypoint in partial-eval mode: BFCL's AST checker
     compares the model's function call against the category's possible-value
     ground truth and writes a per-category score file.

Reward = that category's `accuracy` (for a single task, 0.0 or 1.0 — the AST
checker's own verdict; benchmarks/RULES.md rule 18 permits a float in
[0.0, 1.0]). `passed` is `reward == 1.0`.

Fail-closed: any error leaves reward = 0.
"""

import os

# BFCL computes its result/score/id-file paths from PROJECT_ROOT at IMPORT time
# (bfcl_eval.constants.eval_config reads $BFCL_PROJECT_ROOT). The installed
# package tree is read-only, so point it at a writable dir BEFORE importing any
# bfcl_eval module.
PROJECT_ROOT = os.environ.setdefault("BFCL_PROJECT_ROOT", "/tmp/bfcl_root")
os.makedirs(PROJECT_ROOT, exist_ok=True)
os.environ.setdefault("HF_HOME", "/tmp/hf")
os.makedirs(os.environ["HF_HOME"], exist_ok=True)

import json  # noqa: E402
import re  # noqa: E402
import sys  # noqa: E402
from pathlib import Path  # noqa: E402
from types import SimpleNamespace  # noqa: E402

REWARD_PATH = "/logs/verifier/reward.txt"
TASKS_JSONL = "/tasks/all.jsonl"


def write_reward(value: str) -> None:
    os.makedirs("/logs/verifier", exist_ok=True)
    with open(REWARD_PATH, "w") as f:
        f.write(value)


def resolve_task(task_id: int):
    """Map the sequential EVAL_TASK_ID to (category, bfcl_id).

    Prefer the fields the shared materializer wrote for this task; fall back to
    seeking the build-time map directly.
    """
    mat_cat = f"/tasks/{task_id}/category.txt"
    mat_id = f"/tasks/{task_id}/bfcl_id.txt"
    if os.path.exists(mat_cat) and os.path.exists(mat_id):
        with open(mat_cat) as f:
            category = f.read().strip()
        with open(mat_id) as f:
            bfcl_id = f.read().strip()
        if category and bfcl_id:
            return category, bfcl_id

    with open(TASKS_JSONL) as f:
        for line in f:
            row = json.loads(line)
            if int(row["id"]) == task_id:
                return row["category"], row["bfcl_id"]
    raise SystemExit(f"[runner] task id {task_id} not found in {TASKS_JSONL}")


def register_model(registry_key: str, api_model: str) -> None:
    """Inject a ModelConfig so BFCL will drive `api_model` via the gateway.

    The handler reads OPENAI_BASE_URL / OPENAI_API_KEY from the environment and
    sends `api_model` as the OpenAI `model` field; the gateway resolves it to
    the real upstream. is_fc_model=True selects native function-calling
    (tool_calls), which is what the AST checker expects to parse.
    """
    from bfcl_eval.constants import model_config as mc
    from bfcl_eval.model_handler.api_inference.openai_completion import (
        OpenAICompletionsHandler,
    )

    if registry_key in mc.MODEL_CONFIG_MAPPING:
        return
    mc.MODEL_CONFIG_MAPPING[registry_key] = mc.ModelConfig(
        model_name=api_model,
        display_name=f"{api_model} (gateway, FC)",
        url="http://gateway",
        org="eval-containers",
        license="proprietary",
        model_handler=OpenAICompletionsHandler,
        input_price=None,
        output_price=None,
        is_fc_model=True,
        # BFCL function names contain dots (e.g. `math.factorial`), which the
        # function-calling API forbids, so BFCL sends them as underscores. This
        # flag converts the model's underscore names back to dots before the AST
        # checker grades them against the dotted ground truth. Without it, every
        # dotted-name task fails on the name alone regardless of correct args.
        underscore_to_dot=True,
    )


def read_accuracy(category: str) -> float:
    """Read the AST checker's per-category accuracy from the score file."""
    from bfcl_eval.constants.category_mapping import VERSION_PREFIX

    score_root = Path(PROJECT_ROOT) / "score"
    target = f"{VERSION_PREFIX}_{category}_score.json"
    matches = list(score_root.rglob(target))
    if not matches:
        raise FileNotFoundError(f"no score file {target} under {score_root}")
    # The first line of a BFCL score file is the summary header dict.
    with matches[0].open() as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            header = json.loads(line)
            return float(header["accuracy"])
    raise ValueError(f"empty score file {matches[0]}")


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
    api_model = os.environ.get("MODEL") or os.environ.get("EVAL_MODEL") or "eval-model"
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    temperature = float(os.environ.get("BFCL_TEMPERATURE", "0.001"))
    # A registry key is a filesystem-safe handle BFCL uses for result/score
    # dir names; the API model string (possibly with "/") is sent on the wire.
    registry_key = re.sub(r"[^0-9A-Za-z._-]", "_", api_model)

    category, bfcl_id = resolve_task(task_id)
    print(
        f"[runner] task={task_id} category={category} bfcl_id={bfcl_id}",
        file=sys.stderr,
    )
    print(f"[runner] model={api_model} base_url={base_url}", file=sys.stderr)

    # The key is a placeholder; the real credential lives on the gateway.
    os.environ.setdefault("OPENAI_API_KEY", "sk-proxy")
    os.environ["OPENAI_BASE_URL"] = base_url

    from bfcl_eval._llm_response_generation import main as generation_main
    from bfcl_eval.constants.eval_config import TEST_IDS_TO_GENERATE_PATH
    from bfcl_eval.eval_checker.eval_runner import main as evaluation_main

    register_model(registry_key, api_model)

    # Select exactly this one task for --run-ids generation.
    with open(TEST_IDS_TO_GENERATE_PATH, "w") as f:
        json.dump({category: [bfcl_id]}, f)

    gen_args = SimpleNamespace(
        model=[registry_key],
        test_category=[category],  # ignored under run_ids, but must be valid
        temperature=temperature,
        include_input_log=False,
        exclude_state_log=False,
        num_gpus=1,
        num_threads=1,
        gpu_memory_utilization=0.9,
        backend="sglang",  # unused for an API model
        skip_server_setup=False,
        local_model_path=None,
        result_dir="result",
        allow_overwrite=True,
        run_ids=True,
        enable_lora=False,
        max_lora_rank=None,
        lora_modules=None,
    )

    try:
        generation_main(gen_args)
    except Exception as e:  # noqa: BLE001 — fail-closed, reward stays 0
        print(f"[runner] generation failed: {e}", file=sys.stderr)
        return 1

    try:
        # partial_eval=True: score only the single entry present, tolerating the
        # rest of the category being absent.
        evaluation_main([registry_key], [category], "result", "score", True)
    except Exception as e:  # noqa: BLE001
        # The AST checker writes the per-category score file BEFORE the run's
        # final leaderboard-CSV aggregation, which computes a latency stdev and
        # raises on a single data point (we always run exactly one task). That
        # aggregation is cosmetic, so a crash here is non-fatal: the score file
        # is the graded verdict, and read_accuracy below is the real gate —
        # reward stays 0 only if no score file was actually written.
        print(
            f"[runner] evaluation post-processing raised (non-fatal): {e}",
            file=sys.stderr,
        )

    try:
        accuracy = read_accuracy(category)
    except (OSError, ValueError, KeyError) as e:
        print(f"[runner] could not read score: {e}; reward stays 0", file=sys.stderr)
        return 1

    reward = min(1.0, max(0.0, accuracy))
    write_reward(repr(reward))
    print(
        f"[runner] accuracy={accuracy} reward={reward} passed={reward == 1.0}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
