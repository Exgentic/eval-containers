# EnterpriseOps-Gym

**Status:** released — fixture `tests/replay/fixtures/enterpriseops-gym-0-codex.trajectory.jsonl` from a real codex+gpt-5.4 run (calendar task 0, scored 0.667 / 3 verifiers).

**Paper:** [EnterpriseOps-Gym](https://arxiv.org/abs/2603.13594) (Malay, Nayak et al., ServiceNow AI Research, 2026)
**Upstream:** [ServiceNow/EnterpriseOps-Gym](https://github.com/ServiceNow/EnterpriseOps-Gym) @ `09593147`
**Dataset:** [ServiceNow-AI/EnterpriseOps-Gym](https://huggingface.co/datasets/ServiceNow-AI/EnterpriseOps-Gym) @ `c8e538e`

649 single-agent enterprise tasks across 8 domains (count measured at the pinned dataset SHA; the upstream README quotes a larger headline number that aggregates tool-set modes differently):

| Domain   | Tasks |
|----------|------:|
| calendar | 61    |
| csm      | 103   |
| drive    | 64    |
| email    | 67    |
| hr       | 102   |
| hybrid   | 88    |
| itsm     | 103   |
| teams    | 61    |

Outcome-based scoring: SQL verifiers query final DB state via each MCP server's `/api/sql-runner` endpoint. Deterministic — no LLM-as-judge in the hot path.

## Per-task sidecar selection ([RULES.md](RULES.md) 24h)

Each task's `gym_servers_config` names a subset of the seven sidecars. The chart self-resolves which to render from [`task-profiles/enterpriseops-gym.json`](../_chart/task-profiles/enterpriseops-gym.json); compose reads the same map.

For 86% of tasks (561/649) the subset is a single sidecar — running only what the task needs cuts the per-task memory floor by ~7×.

Distribution:

| Sidecar count | Tasks |
|--------------:|------:|
| 1             | 561   |
| 2             | 88    |

Regenerate the map after bumping `BENCHMARK_VERSION`:

```bash
python3 containers/benchmarks/enterpriseops-gym/gen-task-profiles.py
```

## How it runs

One agent, one chart, one reward — the standard shape.

**k8s (the chart self-resolves the task's sidecars):**
```bash
helm template containers/benchmarks/_chart \
  --set benchmark=enterpriseops-gym \
  --set agent=claude-code \
  --set task=0 | kubectl apply -f -
```

**compose (bare = full set; lean = name the task's sidecars from the same map):**
```bash
# Full set (the standalone default, rule 1)
EVAL_TASK_ID=0 docker compose up

# Lean (only the task's sidecars; runner waits on gateway, not on the rest)
EVAL_TASK_ID=0 docker compose up runner gateway otelcol \
  $(jq -r --arg t "$EVAL_TASK_ID" '.[$t][]' ../_chart/task-profiles/enterpriseops-gym.json)
```

## What scores and what skips

- `database_state` verifiers — full support, deterministic SQL compare. The vast majority.
- `response_check` verifiers — LLM-as-judge against the agent's final response. **Skipped in v1**: counted as `skipped`, not failed, so `reward` reflects only what was evaluated.
- `tool_execution` verifiers — needs the agent's tool-call list in a known shape. **Skipped in v1** for the same reason.

`verifier_report.json` under `/output/task/` enumerates every verifier outcome — pass, fail, or skip with reason.

## Setup contract

`setup_task.py` (root, pre-agent) reads the task's `gym_servers_config`, POSTs `/api/seed-database` on each referenced server to create a fresh per-task DB, captures the resulting `database_id` + auth context in `/var/eval-state/state.json` (root-only, mode 600), and writes the agent's `TASK`: system policy, MCP endpoints + per-server `database_id` + auth headers, allowed tool list, user request.

The agent receives only `TASK` and the model `*_BASE_URL`s (per [RULES.md](RULES.md) §7). It figures out how to speak MCP on its own.

`verify_task.py` (root, post-agent) queries final state and writes `reward = passed/total` plus the full `verifier_report.json`.

## What's still missing

1. **Replay fixture** under `tests/fixtures/` for a small task. Once it lands and the replay sweep is green, flip `LABEL eval.benchmark.released="true"`.
2. **`response_check` and `tool_execution` verifier types.** Both need extra plumbing; `verify_task.py` skips them today.
3. **Supply-chain debt.** The 7 MCP service images are `:latest` tags from a third-party Docker Hub account. `tests/FLEET.md` question 6 flags this yellow until mirrored to `ghcr.io/exgentic/backends/` by digest.

## Why `EVAL_AGENT` is meaningful here

The benchmark deliberately does **not** include its own agent — each row provides a system prompt, a tool catalogue, and a user request; the *agent* is whatever the user plugs in. EnterpriseOps-Gym's own paper measures it against ReAct / PlannerReact / DecomposingPlanner orchestrators; the value of this port is that you can swap in Claude Code, Codex, OpenHands, or any other agent and get a comparable score on the same arena.
