# Runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

An evaluation answers one question: *"How well does this agent solve
this benchmark using this model?"* To answer it, the framework gives
the agent a task, lets it work, then checks the result. This page
explains exactly how that happens inside the container.

## The three pieces

You don't ship one big image. You ship small, independent images that
each do one thing:

- A **benchmark image** knows how to pose a problem and grade the
  answer. It contains tasks, a setup script, and a grading script.
- An **agent image** knows how to solve problems. It contains the
  agent's code and dependencies.
- A **model image** contains a gateway proxy that sits between the agent
  and the LLM provider, logging every API call.

At build time, these three images (plus some runtime tooling) are
combined into a single **evaluation image**. Think of it as layering
transparencies on an overhead projector — each image contributes its
files, and the result has everything needed to run one evaluation.

## What happens when you run an eval

The evaluation image starts, and four things happen in order:

### 1. Setup — prepare the task

The benchmark's setup script (`/entrypoint.sh`) runs first. Its job is
to pick the current task and set a `TASK` environment variable — this is
the plain-text prompt the agent will see. Most benchmarks ship a file
with all their tasks and unpack the one matching `EVAL_TASK_ID`; some
bake one task per image at build time instead.

Once `TASK` is set, the setup script hands control to the framework.

### 2. Agent — solve the task

The agent's script (`/run.sh`) runs next. It sees a deliberately small
environment:

- `TASK` — the problem to solve
- `OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL`
  — all pointing at the gateway proxy, never at a real provider
- `MODEL` — which model to talk to
- `TIMEOUT` — how long it has

The agent runs as an unprivileged user and cannot read the grading
script's test data, the task answers, or the gateway configuration.
This is enforced by file permissions — the agent's user simply cannot
access those paths.

### 3. Grade — score the output

The benchmark's grading script (`/grade.sh`) runs after the agent
finishes. It reads whatever the agent produced and writes a score to
`/logs/verifier/reward.txt` — a number between 0 and 1. How it decides
that score is entirely up to the benchmark: string comparison, a test
suite, a judge LLM, or something custom.

### 4. Result — record the outcome

A framework utility (`write-result`) reads the score and writes
structured JSON files to `/output/` — one for the task result, one for
the agent metadata, one for the model. This is what the outside world
reads to know what happened.

## The contract — what your image must provide

The build that combines these images copies specific files from each
one. If your image is missing a required file, the build breaks.

**If you're writing a benchmark**, you provide:

- `/entrypoint.sh` — sets `TASK`, then hands off with `exec "$@"`
- `/grade.sh` — scores the agent's output, writes to
  `/logs/verifier/reward.txt`

**If you're writing an agent**, you provide:

- `/run.sh` — your agent's launch script
- `/opt/agent/` — your agent's installation (including `install.sh`)

Everything else — the process orchestrator, the result writer, the
telemetry collector — comes from the framework. You don't need to think
about it.

## Runtime modes

The same evaluation image runs in three modes. The four steps above
always happen; the mode decides where each process lives.

**Single-image** — everything in one container. A built-in orchestrator
(process-compose) runs the gateway, agent, grader, and result writer as
separate processes inside the same container.

**Compose** — three containers (`otelcol`, `gateway`, `runner`). The
gateway and telemetry collector get their own containers; the runner
still orchestrates the agent → grade → result chain internally.

**Kubernetes** — a Helm Job. The gateway and telemetry run as Kubernetes
sidecars; the runner pod handles agent → grade → result, then tears down
the sidecars when done.

See [Triple-mode](triple-mode.md) for the full details on each.

## Not every benchmark follows the standard flow

The entrypoint → framework → orchestrator chain is the default path, not
a hard requirement. A benchmark with unusual needs can override it.

For example, tau-bench replaces the runner's entrypoint entirely with
its own Python script and adds extra containers for its harness. It
doesn't use the built-in orchestrator at all — but the four steps
(setup → agent → grade → result) still happen in order.

## Where to go next

- [Overview](overview.md) — what Eval Containers is
- [Triple-mode](triple-mode.md) — more on the three runtime modes
- [Add a benchmark](../guides/add-a-benchmark.md) — build one yourself
