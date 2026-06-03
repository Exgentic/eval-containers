# Run your first eval

*Guide · for operators · derives from [`README.md`](../../README.md), [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md).*

This runs one AIME task with the Codex agent against `gpt-5.4`, locally.

## 1. Prerequisites

[Install](install.md) Docker Compose ≥ 2.34, build the CLI, and set your key:

```bash
echo "OPENAI_API_KEY=sk-..." > .env
```

## 2. Run a task

With the repo cloned, use `--local` to run from on-disk artifacts:

```bash
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4 --local
```

This maps to a plain Docker command — print it without running via `--dry-run`:

```bash
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4 --local --dry-run
# → EVAL_BENCHMARK=aime EVAL_AGENT=codex EVAL_MODEL=gpt-5.4 EVAL_TASK_ID=0 \
#     docker compose -f ./benchmarks/aime/compose.yaml up --abort-on-container-exit
```

You can run that `docker compose` line yourself — the CLI is just a reminder of
it.

## 3. Read the result

```bash
cat output/aime/0/task/result.json
```

The primary metric is `reward`; benchmarks may add named fields alongside it
(e.g. `passed`, `exact_match`). The model trajectory is recorded independently
of the agent at:

```bash
cat output/aime/0/model/trajectory.jsonl
```

## Variations

```bash
# A different agent / model
eval-containers run aime --task-id 0 --agent claude-code --model gpt-5.4 --local

# Cap spend (USD) and set a timeout
eval-containers run aime --task-id 0 --agent codex --max-budget 1 --timeout 600 --local

# A different runtime (see Triple-mode)
eval-containers run aime --task-id 0 --agent codex --mode container --local
```

Every `EVAL_*` variable has a matching `--kebab-case` flag — full list in the
[CLI reference](../reference/cli.md) and [Environment variables](../reference/env-vars.md).

## Next

- Scale out: [Deploy on Kubernetes](deploy-on-kubernetes.md)
- Understand the wiring: [Isolation & gateways](../concepts/isolation-and-gateways.md)
