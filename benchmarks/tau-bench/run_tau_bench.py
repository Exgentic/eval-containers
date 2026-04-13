"""Run TAU-bench with the LLM bridge as the agent endpoint.

TAU-bench uses two LLMs:
1. Agent LLM: makes tool-calling decisions (routed through bridge -> real agent)
2. User simulator LLM: plays the customer role (direct to model proxy)

This script runs a single task identified by TASK_ID.

The trick: both use litellm.completion() with the openai provider, which reads
OPENAI_BASE_URL. We set that to the bridge for the agent, then monkey-patch the
user simulator to call the model proxy directly.
"""

import os
import sys
import json

BRIDGE_URL = os.environ.get("BRIDGE_URL", "http://bridge:8000/v1")
MODEL_URL = os.environ.get("MODEL_URL", "http://model:4000/v1")
TASK_ID = int(os.environ.get("TASK_ID", "0"))
MODEL_NAME = os.environ.get("MODEL_NAME", "gpt-4o")

# Set OPENAI_BASE_URL to bridge — this is what the agent's litellm calls will use
os.environ["OPENAI_BASE_URL"] = BRIDGE_URL
os.environ["OPENAI_API_KEY"] = os.environ.get("OPENAI_API_KEY", "sk-proxy")

# Read task metadata
domain = open(f"/tasks/{TASK_ID}/domain.txt").read().strip()

print(f"[runner] task={TASK_ID} domain={domain}", file=sys.stderr)
print(f"[runner] bridge={BRIDGE_URL} model={MODEL_URL}", file=sys.stderr)

# Monkey-patch the user simulator to use the model URL directly.
# The agent's litellm calls go through OPENAI_BASE_URL (bridge),
# but the user simulator should talk to the model proxy directly.
import litellm

_original_completion = litellm.completion


def _patched_completion(*args, **kwargs):
    """Route user simulator calls to model, agent calls to bridge."""
    # User simulator calls don't have tools; agent calls do
    has_tools = kwargs.get("tools") is not None and len(kwargs.get("tools", [])) > 0
    if not has_tools:
        # User simulator call — route to model directly
        kwargs["base_url"] = MODEL_URL
    # Otherwise, agent call goes through OPENAI_BASE_URL (bridge)
    return _original_completion(*args, **kwargs)


litellm.completion = _patched_completion

# Now import and run tau-bench
from tau_bench.run import run
from tau_bench.types import RunConfig

# Find the task index within the domain's task list
# TASK_ID is our sequential index; we need to find the matching index within
# the domain's own task list
task_index_in_domain = None
idx = 0
if domain == "retail":
    from tau_bench.envs.retail.tasks_test import TASKS_TEST as retail_tasks
    task_index_in_domain = TASK_ID  # retail tasks come first (0..114)
elif domain == "airline":
    from tau_bench.envs.retail.tasks_test import TASKS_TEST as retail_tasks
    task_index_in_domain = TASK_ID - len(retail_tasks)  # airline tasks follow

print(f"[runner] domain_task_index={task_index_in_domain}", file=sys.stderr)

config = RunConfig(
    env=domain,
    model=MODEL_NAME,
    user_model=MODEL_NAME,
    model_provider="openai",
    user_model_provider="openai",
    task_ids=[task_index_in_domain],
    log_dir="/logs/tau-bench",
    num_trials=1,
    temperature=0.0,
    agent_strategy="tool-calling",
    task_split="test",
)

try:
    results = run(config)

    # Log results
    for r in results:
        print(
            f"[runner] task_id={r.task_id} reward={r.reward}",
            file=sys.stderr,
        )

    # Write reward from tau-bench's own evaluation
    os.makedirs("/logs/verifier", exist_ok=True)
    if results:
        reward = results[0].reward
        with open("/logs/verifier/reward.txt", "w") as f:
            f.write(str(float(reward)))
        print(f"[runner] wrote reward={reward}", file=sys.stderr)
    else:
        with open("/logs/verifier/reward.txt", "w") as f:
            f.write("-1")

    # Save full results
    os.makedirs("/logs/tau-bench", exist_ok=True)
    with open("/logs/tau-bench/results.json", "w") as f:
        json.dump([r.model_dump() if hasattr(r, "model_dump") else str(r) for r in results], f, indent=2)

except Exception as e:
    print(f"[runner] error: {e}", file=sys.stderr)
    import traceback
    traceback.print_exc(file=sys.stderr)
    os.makedirs("/logs/verifier", exist_ok=True)
    with open("/logs/verifier/reward.txt", "w") as f:
        f.write("-1")
    sys.exit(1)
