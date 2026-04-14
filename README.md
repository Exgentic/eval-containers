# Dock

Container-native AI agent evaluations. 96 benchmarks, 17 agents, one `docker compose up`.

## Quick start

```bash
# Set your API key
echo "OPENAI_API_KEY=sk-..." > .env

# Run one task — pure docker, no clone, no CLI
DOCK_BENCHMARK=aime DOCK_TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-5.4 \
  docker compose -f oci://quay.io/dock-eval/evaluate up --abort-on-container-exit

# Results
cat output/aime/0/task/result.json
```

One URL for every evaluation. Benchmark, agent, model, and task are all `DOCK_*` env vars.

Requires Docker Compose ≥ 2.34 for `oci://` support. See [offline / older Docker](#offline--older-docker) below for alternatives.

## Or use the `dock` CLI

Same thing, fewer keystrokes:

```bash
dock run --benchmark aime --task-id 0 --agent codex --model gpt-5.4
```

Every `DOCK_*` env var has a matching `--kebab-case` flag. Pick whichever you prefer.

## Environment variables

All Dock env vars are prefixed `DOCK_` to avoid collision with CI systems, orchestrators, and user scripts.

**Axis selection**

| Variable | Meaning | Default |
|---|---|---|
| `DOCK_BENCHMARK` | Which benchmark to run | — |
| `DOCK_AGENT` | Which agent to run | — |
| `DOCK_MODEL` | Which model to route calls to | — |
| `DOCK_TASK_ID` | Which task within the benchmark | `0` |

**Container versions** (which image tag to pull)

| Variable | Meaning | Default |
|---|---|---|
| `DOCK_BENCHMARK_TAG` | Benchmark container version | `latest` |
| `DOCK_AGENT_TAG` | Agent container version | `latest` |
| `DOCK_MODEL_TAG` | Model container version | `latest` |

**Internal software versions** (what runs inside the container)

| Variable | Meaning | Default |
|---|---|---|
| `DOCK_BENCHMARK_VERSION` | Dataset revision inside the benchmark | built-in pin |
| `DOCK_AGENT_VERSION` | Upstream CLI version inside the agent | built-in pin |
| `DOCK_LITELLM_VERSION` | LiteLLM version inside the model | built-in pin |

**Runtime**

| Variable | Meaning | Default |
|---|---|---|
| `DOCK_TIMEOUT` | Agent timeout in seconds | `300` |
| `DOCK_REGISTRY` | Registry to pull from | `quay.io/dock-eval` |

Container tags are Docker-native (different tag → different pull). Internal versions are runtime overrides (the entrypoint installs the requested version at container start).

Every image ships with a **reproducible default**, so casual users never touch the version vars. Power users pin.

## Concepts

- **Benchmark** — a collection of tasks (AIME has 90, SWE-bench has 500)
- **Task** — a single problem within a benchmark
- **Agent** — the AI system attempting the task (Claude Code, Codex, OpenHands, SWE-agent, Plandex, ...)
- **Model** — the LLM the agent calls, routed through a logging proxy. Works with any [LiteLLM-supported provider](https://docs.litellm.ai/docs/providers) (OpenAI, Anthropic, Google, Azure, Ollama, and 100+ more).
- **Evaluation** — one benchmark + one agent + one model, defined by one Compose artifact.

## Offline / older Docker

If you're on Docker < 2.34, airgapped, or just prefer a local file:

```bash
# Fetch + flatten the compose file once (needs a machine with network)
DOCK_BENCHMARK=aime DOCK_AGENT=codex DOCK_MODEL=gpt-5.4 \
  docker compose -f oci://quay.io/dock-eval/evaluate config > aime.compose.yaml

# Transport aime.compose.yaml anywhere. Run offline:
DOCK_TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-5.4 \
  docker compose -f aime.compose.yaml up --abort-on-container-exit
```

Or for fully airgapped deployments, bundle the images too:

```bash
docker save quay.io/dock-eval/evals/aime--codex:latest \
            quay.io/dock-eval/models/gpt-5.4:latest \
  | gzip > aime-bundle.tar.gz
```

## Local development

If you have the repo cloned and want to iterate on a benchmark or agent without pushing to the registry:

```bash
dock run --benchmark aime --task-id 0 --agent codex --model gpt-5.4 --local
```

`--local` points at `benchmarks/<name>/compose.yaml` on disk instead of `oci://...`.

## Rules

All work is governed by RULES documents. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full index.

| Rules | Scope |
|-------|-------|
| [RULES.md](RULES.md) | Core principles |
| [benchmarks/RULES.md](benchmarks/RULES.md) | Building benchmarks |
| [agents/RULES.md](agents/RULES.md) | Building agents |
| [models/RULES.md](models/RULES.md) | Building models |
| [src/RULES.md](src/RULES.md) | CLI |
| [compose/RULES.md](compose/RULES.md) | Naming, compose, output, registry |

## Setup

- [tests/LOCAL.md](tests/LOCAL.md) — local dev loop (Docker Desktop, Podman, Rosetta)
- [RELEASE.md](RELEASE.md) — how CI builds and publishes the fleet
