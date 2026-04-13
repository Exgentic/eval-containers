# Dock

A build system for AI agent evaluations. Produces Docker images and Compose files. Run them anywhere with `docker compose up`.

## Quick Start

```bash
# Set your API key
echo "OPENAI_API_KEY=sk-..." > .env

# Run one task
TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-5.4 \
  docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit

# Results
cat output/aime/0/task/result.json
```

## Concepts

- **Benchmark** — a collection of tasks (AIME has 90, SWE-bench has 500)
- **Task** — a single problem within a benchmark
- **Agent** — the AI system attempting the task (Claude Code, Codex, OpenHands)
- **Model** — the LLM the agent calls, routed through a logging proxy. Works with any [LiteLLM-supported provider](https://docs.litellm.ai/docs/providers) (OpenAI, Anthropic, Google, Azure, Ollama, and 100+ more)
- **Evaluation** — one task + one agent + one model, defined by one Compose file

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
