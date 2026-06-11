# Overview

*Concept · for everyone · derives from [`.agents/RULES.md`](../../.agents/RULES.md), [`.agents/MANIFESTO.md`](../../.agents/MANIFESTO.md).*

Eval Containers is a build system for AI-agent evaluations. It turns each
benchmark, agent, and model into a versioned container image, and runs an
evaluation as a plain `docker compose` / `kubectl` invocation — no framework,
no daemon, no lock-in.

## The one idea

**An evaluation is one benchmark + one agent + one model.** These are three
independent axes: any benchmark runs against any agent against any model, and
each is swappable without touching the others.

```
  benchmark   ×   agent   ×   model        =   one evaluation
  (aime)          (codex)     (gpt-5.4)         aime · codex · gpt-5.4
```

A single problem inside a benchmark is a **task** (selected by `EVAL_TASK_ID`).

## Why containers

- **The image is the product.** Everything Eval Containers ships is a Docker
  image or a Compose file — immutable, versioned, portable. If you can
  `docker pull` and `docker compose up`, you can run any evaluation.
- **Standalone artifacts.** Every published image works without Eval Containers
  installed. Delete this repo and the artifacts still run.
- **No framework lock-in.** Evaluations run on plain Docker / Kubernetes. There
  is no runtime to install.

## How a run is wired

An evaluation runs three units together:

- **runner** — the benchmark + agent; materializes the task, runs the agent, grades the result.
- **gateway** — a logging proxy in front of the model. Every LLM call is
  recorded here, independent of the agent (see [Isolation & gateways](isolation-and-gateways.md)).
- **otelcol** — collects telemetry.

The result lands as `result.json`, with the primary metric named `reward`.

## Two version axes

Every image has a reproducible default and two independent version knobs:

- **Container version** — *which image to pull* — set by the image **tag**
  (`EVAL_BENCHMARK_TAG`, `EVAL_AGENT_TAG`, `EVAL_MODEL_TAG`).
- **Internal version** — *what runs inside* — set at runtime
  (`EVAL_BENCHMARK_VERSION`, `EVAL_AGENT_VERSION`, `EVAL_LITELLM_VERSION`).

Casual users never touch these; power users pin. Full list in
[Environment variables](../reference/env-vars.md).

## Where to go next

- [Triple-mode](triple-mode.md) — the three runtimes for the same eval
- [Run your first eval](../guides/run-your-first-eval.md) — hands-on
