# automationbench

AutomationBench (Zapier) — business-workflow agent tasks over simulated SaaS
tools, state-verified.

**Status:** Not yet released — no replay fixture landed (see
[`.agents/benchmarks/RULES.md`](../../../.agents/benchmarks/RULES.md) rule 21a).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 600 (6 public domains × 100) |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Evaluation | **model-only** (native harness is the agent scaffold) |
| Upstream | [github.com/zapier/AutomationBench](https://github.com/zapier/AutomationBench) |
| License | MIT |
| Dataset revision | `4a8e1061254004d9dac807054eed33fad7d1ff14` (commit — no upstream tags) |
| Canonical agent | `claude-code` (naming only; the harness ignores the agent axis) |

## What this is

AutomationBench evaluates an agent on realistic business workflows across ~47
**simulated** SaaS tools (Salesforce, Gmail, Slack, HubSpot, Zendesk, …) in six
domains (sales, marketing, operations, support, finance, hr). The agent makes
tool calls that mutate an in-process `WorldState`; grading compares the final
world state to per-task assertions. Everything is in-process — no live external
services, no LLM-as-judge.

## Model-only, native harness

Unlike a print-an-answer benchmark, AutomationBench drives the model through its
**own** tool-calling loop (the `auto-bench` harness). We run that harness
directly, pointing its model endpoint at the fleet gateway. This is a
**model-only** evaluation: the harness *is* the agent scaffold, so there is no
repo agent, and — because AutomationBench has no user simulator — none of the
bridge / second-gateway complexity that `tau-bench` needs.

The runner (which holds `EVAL_TASK_ID`, withheld from the scrubbed agent phase
per rule 7) resolves the sequential task id to its upstream `task_name` via a
build-time map (`/tasks/all.jsonl`), then runs:

```
auto-bench --tasks <task_name> --api chat_completions \
  --base-url $OPENAI_BASE_URL --api-key-var OPENAI_API_KEY \
  --model $MODEL --domains public --max-steps 50 \
  --export-json /output/task/automationbench.json
```

## How it's graded

Deterministic, in-process assertion checking by AutomationBench's own rubric.
The reward is **`task_completed_correctly`** (0/1): `1` iff every scored
assertion passes. `run_automationbench.py` reads the exported per-task `passed`
flag and writes `1`/`0` to `/logs/verifier/reward.txt`; the shared
`write-result` then emits `result.json`.

## Files

- `Dockerfile` — builds the benchmark image (`FROM python:3.13-slim`; installs
  AutomationBench at the pinned commit; builds the task-name map).
- `run_automationbench.py` — single-task runner (resolve id → name, run harness,
  write reward).
- `compose.yaml` — compose-mode deployment (runner entrypoint override; no
  bespoke services).
- single — the standalone bundle, from the generic `core/standalone.Dockerfile`.
- k8s — the shared chart `benchmarks/_chart`, `--set benchmark=automationbench`
  (`presets/automationbench.yaml` for the harness entrypoint + longer timeout).
- `README.md` — this file.
- `AUDIT.md` — standing audit record.

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running — three deployment surfaces

| Mode | File | Invocation |
|------|------|------------|
| **single** | `core/standalone.Dockerfile` | `docker run -e OPENAI_API_KEY=… -e OPENAI_API_BASE=… <image>-standalone` |
| **compose** | `compose.yaml` | `docker compose -f benchmarks/automationbench/compose.yaml up` |
| **k8s** | shared chart | `helm template automationbench benchmarks/_chart --set benchmark=automationbench \| kubectl apply -f -` |

```bash
# Compose mode
OPENAI_API_KEY=… OPENAI_API_BASE=… \
  docker compose -f benchmarks/automationbench/compose.yaml up

# A different task (rule 24c — parameterized via ${TASK_ID:-0})
TASK_ID=42 docker compose -f benchmarks/automationbench/compose.yaml up

# k8s, a different task
helm template automationbench benchmarks/_chart \
  --set benchmark=automationbench --set task=42 | kubectl apply -f -
```

## Build args

To rebuild the eval image from source (instead of pulling):

```bash
docker build -f core/combination.Dockerfile \
  --build-arg BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/automationbench:latest \
  --build-arg AGENT_IMAGE=ghcr.io/exgentic/agents/claude-code:latest \
  --build-arg AGENT_VERSION=2.1.0 \
  --build-arg MODEL_IMAGE=ghcr.io/exgentic/models/bifrost:latest \
  -t ghcr.io/exgentic/evals/automationbench--claude-code:latest .
```
