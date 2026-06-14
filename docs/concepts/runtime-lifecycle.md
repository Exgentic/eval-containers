# Build and runtime lifecycle

*Concept · for benchmark and agent authors · derives from [`.agents/benchmarks/RULES.md`](../../.agents/benchmarks/RULES.md) rule 12.*

An evaluation answers one question: *"How well does this agent solve
this benchmark using this model?"* To answer it, the framework builds
a container image that has everything inside — the benchmark's work
environment, the agent, and a gateway proxy — then runs it. This page
explains both steps: what gets built, and what happens when you run it.

## Build — assemble the evaluation image

You don't ship one big image. You ship small, independent images that
each do one thing:

- A **benchmark image** is the work environment. It contains the task
  data and any software or services the agent will need — files,
  databases, websites, sidecars. It also provides an entrypoint script
  that materializes the specific task and a grading script that scores
  the agent's work.
- An **agent image** contains the agent's code and dependencies.
- A **model image** contains a gateway proxy that sits between the agent
  and the LLM provider, logging every API call.

At build time, these three images (plus some runtime tooling) are
stitched into a single **evaluation image**. Think of it as layering
transparencies on an overhead projector: each image contributes its
files, and the result is one container with everything inside. The
agent is installed, the gateway is in place, the benchmark's files and
software are all there. Nothing gets installed at runtime.

### The contract — what your image must provide

The build copies specific files from each image. If your image is
missing a required file, the build breaks.

**If you're writing a benchmark**, you provide:

- `/entrypoint.sh` — materializes the task (sets `TASK`), then hands
  off with `exec "$@"`
- `/grade.sh` — scores the agent's output, writes to
  `/logs/verifier/reward.txt`

**If you're writing an agent**, you provide:

- `/run.sh` — your agent's launch script
- `/opt/agent/` — your agent's installation (including `install.sh`)

Everything else — the process orchestrator, the result writer, the
telemetry collector — comes from the framework. You don't need to think
about it.

## Runtime — what happens when you run an eval

The container starts, and four things happen in order:

### 1. Launch — materialize the task

The benchmark's entrypoint (`/entrypoint.sh`) runs first. Its job is to
materialize the specific task: it sets a `TASK` environment variable
with the plain-text prompt the agent will see, then hands control to
the framework. Most benchmarks ship a file with all their tasks and
unpack the one matching `EVAL_TASK_ID`; some bake one task per image
at build time instead.

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
(launch → agent → grade → result) still happen in order.

## Where to go next

- [Overview](overview.md) — what Eval Containers is
- [Triple-mode](triple-mode.md) — more on the three runtime modes
- [Add a benchmark](../guides/add-a-benchmark.md) — build one yourself
