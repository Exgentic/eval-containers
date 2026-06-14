---
name: add-benchmark
description: >-
  Use when adding a new benchmark image to the fleet — a Dockerized task plus
  verifier that evaluates an agent. Walks the directory layout, the five
  authored files, the two version knobs, required labels, and the three
  deployment surfaces. For packaging the AI system that runs *inside* a
  benchmark (install + entrypoint, LLM access), use add-agent instead.
---

# Add a benchmark

A benchmark image contains everything needed to evaluate an agent on a task:
the task data, the test logic, and the entrypoints. This skill produces a new
benchmark that satisfies `.agents/benchmarks/RULES.md`. Read that RULES.md
before starting. A copyable starting point lives at
[`assets/TEMPLATE.md`](assets/TEMPLATE.md) — the steps below tell you how to use
it and why each piece exists.

## Before you start: pick the benchmark shape

Decide which of the three task models fits, because it changes how `EVAL_TASK_ID`
flows and how the image is built:

- **Shared-env** — one image, many tasks; all tasks share the same environment
  and only the instruction differs (AIME, SimpleQA, GPQA). `EVAL_TASK_ID` is the
  *only required runtime input* and selects the task at run time.
- **Per-task, prebuilt base** — one image per task and a per-task upstream base
  image already exists; `EVAL_TASK_ID` is a *build-time* `ARG` and a single
  `Dockerfile` bakes exactly one task (see
  `benchmarks/swe-bench/Dockerfile`).
- **Per-task, built-from-source** — one image per task, but **no** per-task
  upstream image exists and each task ships its own `environment/Dockerfile`
  (heterogeneous bases + setup — the Harbor task format). Ship an executable
  **`build.sh <image> <task-id>`** that (1)
  builds the task's own `environment/Dockerfile` into a task-env image, then (2)
  overlays the eval pipeline via a `FROM ${TASK_BASE}` `Dockerfile`. Do NOT
  improvise a shared base that clones the whole upstream repo — that is the
  anti-pattern rule 24g exists to prevent. Copy `benchmarks/terminal-bench/`
  (`build.sh` + overlay `Dockerfile` + fetch-the-gold `solution.sh`) and
  substitute the repo + ref (`.agents/benchmarks/RULES.md:24g`).

Either `docker run` (single-image) or `docker compose up` (multi-service) MUST
work with no Dock install and no internet, resolving task content, expected
answer, and any attached files from `EVAL_TASK_ID` alone
(`.agents/benchmarks/RULES.md:1`, `.agents/benchmarks/RULES.md:2`).

## Steps

1. **Create the benchmark directory.** Make `benchmarks/<name>/` and copy
   [`assets/TEMPLATE.md`](assets/TEMPLATE.md) as your scaffold. Every benchmark
   ships four authored files: `Dockerfile` (builds the base image with
   tasks + verifier), the per-benchmark deploy files `container.Dockerfile` +
   `compose.yaml`, and `README.md`. The k8s surface is the shared chart selected
   with `--set benchmark=<name>` — author a `benchmarks/_chart/presets/<name>.yaml`
   only if the benchmark needs bespoke topology. *Why:* the directory is the
   unit a CI test walks; missing `container.Dockerfile` or `compose.yaml` makes
   the benchmark incomplete (`.agents/benchmarks/RULES.md:24`, `.agents/benchmarks/RULES.md:29`).

2. **Write the `Dockerfile` to materialize tasks as flat files.** Fetch the
   dataset and write each task as `/tasks/<id>/problem.txt` + `answer.txt`
   (+ `id.txt`), then `chmod -R 600 /tasks` so the agent UID cannot read them.
   *Why:* tasks SHOULD be plain files, no databases or archives
   (`.agents/benchmarks/RULES.md:16`); shared-env tasks MUST be addressable by
   sequential integers with the upstream id preserved in `id.txt`
   (`.agents/benchmarks/RULES.md:15`); and the agent MUST NOT be able to read
   anything the test phase uses (`.agents/benchmarks/RULES.md:5`,
   `.agents/benchmarks/RULES.md:28`). If a task has attached files, copy them
   into `/app/` (agent-readable) in the entrypoint — never loosen `/tasks/`
   permissions (`.agents/benchmarks/RULES.md:17`). For a **built-from-source**
   benchmark you do not materialize tasks here — the task's own
   `environment/Dockerfile` provides them and the overlay `Dockerfile` adds only
   the agent-readable `instruction.md` plus a **root-only** copy of the tests
   (`chmod 700`, root-owned). Never bake the upstream repo into the image: it
   carries every task's gold solution and tests, which the agent could then read
   (`.agents/benchmarks/RULES.md:5`, `.agents/benchmarks/RULES.md:9`).

3. **Pin the dataset version (knob 1 of 2 — build-time default).** Set
   `ARG DATA_REVISION=<sha>` (or equivalent) as the default and record it in the
   `eval.benchmark.data_revision` label. *Why:* the image MUST produce identical
   task content on every build when no env vars are set
   (`.agents/benchmarks/RULES.md:3`). Get the sha with
   `curl -s https://huggingface.co/api/datasets/<dataset> | jq .sha`.

4. **Honor the runtime version override (knob 2 of 2 — `EVAL_BENCHMARK_VERSION`).**
   The entrypoint MUST read `EVAL_BENCHMARK_VERSION` and, when set, fetch and
   materialize that dataset revision into `/tasks/` in place of the default,
   writing the resolved revision to `/output/task/version.json` before the agent
   runs. When unset, the build-time default applies unchanged. *Why:* this is
   how a caller evaluates against a specific dataset revision without rebuilding
   (`.agents/benchmarks/RULES.md:4`). Note `EVAL_BENCHMARK_TAG` selects the
   image tag to pull — that is Docker's job, not the entrypoint's.

5. **Reuse the shared entrypoint and verifier; do not reimplement them.** `COPY`
   in `core/entrypoint`'s `dock-entrypoint.sh` and, for exact-match scoring,
   `core/test-exact-match`. Your thin `/entrypoint.sh` only assembles `TASK`
   and `EXPECTED_ANSWER` from `/tasks/$TASK_ID/`, then `exec`s the shared
   entrypoint. *Why:* execution MUST follow the three-phase flow agent → test →
   result via the shared entrypoint, and benchmarks MUST NOT bypass it or
   reimplement shared logic (`.agents/benchmarks/RULES.md:12`,
   `.agents/benchmarks/RULES.md:22`). Do NOT bake agent-specific tools
   (browsers, SDKs) into the benchmark image — the agent's `install.sh` brings
   its own (`.agents/benchmarks/RULES.md:23`).

6. **Keep the agent unprivileged, credential-free, and offline by default.** The
   agent process runs as a non-root user (the test phase MAY be root), cannot
   read the LLM API key, and has no outbound internet unless the benchmark
   explicitly needs it. *Why:* `.agents/benchmarks/RULES.md:13` (agent as
   non-root), `.agents/benchmarks/RULES.md:8` (no agent access to credentials),
   `.agents/benchmarks/RULES.md:9` (no agent internet by default). Use the
   simplest isolation that works — file permissions over extra containers
   (`.agents/benchmarks/RULES.md:6`), and standard Docker features only
   (`.agents/benchmarks/RULES.md:11`). Only `TASK`, `EVAL_TASK_ID`,
   `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent; benchmark
   internals MUST NOT leak (`.agents/benchmarks/RULES.md:7`). Agent execution
   MUST be bounded by `EVAL_TIMEOUT`, which the shared entrypoint enforces — the
   benchmark does not implement its own timeout (`.agents/benchmarks/RULES.md:14`).

7. **Wire the scorer to the reward contract.** The test script MUST write a
   float in `[0.0, 1.0]` to `/logs/verifier/reward.txt` (or `-1` for externally
   graded benchmarks). Use the simplest correct scorer — exact match when
   possible, code execution for programming, LLM-as-judge only when nothing
   simpler works. *Why:* `.agents/benchmarks/RULES.md:18` (reward contract),
   `.agents/benchmarks/RULES.md:19` (simplest correct scorer). If grading needs
   an outside service, still collect the agent's output, write `-1`, and do NOT
   approximate the external grader (`.agents/benchmarks/RULES.md:20`). For a
   custom scorer, replace the `test-exact-match` COPY with your own
   `/grade.sh`. Then add the **oracle** that validates this scorer — a
   `benchmarks/<name>/solution.sh`, mounted read-only at oracle run time and never
   `COPY`'d into the image — that *derives* the gold (runs the upstream reference
   solution, or fetches the dataset's canonical answer) and writes it where the
   agent would; never hardcode a literal answer or copy the test's expected-output
   file — a derived oracle stays valid as the data revision moves and proves the
   task is solvable (`.agents/benchmarks/RULES.md:20a`). For a built-from-source benchmark the
   `solution.sh` fetches the per-task upstream gold fresh and MUST read the task
   name from a baked `ENV`, not `EVAL_TASK_ID` (which the oracle overrides to `0`)
   (`.agents/benchmarks/RULES.md:24i`). Validate both halves with
   `eval-containers oracle <name> [--task-id <t>] --local`: a correct gold MUST
   score `1.0` and a no-op MUST score `< 1.0`.

8. **Set the required labels.** Every benchmark image MUST carry `eval.type`,
   `eval.benchmark.name`, `eval.benchmark.description`, `eval.benchmark.tasks`,
   `eval.benchmark.env`, and `eval.benchmark.internet`
   (`.agents/benchmarks/RULES.md:21`). If the `FROM` points at a third-party
   registry outside Dock's control, also declare
   `eval.benchmark.upstream_base="<full image ref>"` so the external dependency
   is visible to audit tools (`.agents/benchmarks/RULES.md:21b`). Do NOT add
   `eval.benchmark.released="true"` yet — that label is earned at the release
   gate (step 11).

9. **Author the three deployment surfaces with one shared env contract.** The
   surfaces MUST share the same env contract (`EVAL_MODEL`, `EVAL_TASK_ID`,
   upstream credentials) and produce byte-equivalent `task/result.json` for the
   same inputs (`.agents/benchmarks/RULES.md:24`):
   - `container.Dockerfile` (**single**) — a *single-line* registry pin
     `FROM <registry>/evals/<name>--<agent>:<tag>`, nothing more (shared-env). A
     **per-task** benchmark instead writes exactly two lines — `ARG EVAL_TASK_ID`
     then `FROM <registry>/evals/<name>-${EVAL_TASK_ID}--<agent>:<tag>` — so the
     pin resolves per task; never hardcode one task's name in the `FROM`. Record
     the canonical build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`,
     `MODEL_IMAGE`) in the `README.md` so CI can rebuild via
     `core/combination.Dockerfile`. Inert `ARG` lines the `FROM` does not
     consume are forbidden (`.agents/benchmarks/RULES.md:24a`).
   - `compose.yaml` (**compose**) — pull in `compose/services.yaml` via
     `include:` and only declare overrides; do NOT inline a service, network, or
     volume that already exists there
     (`.agents/benchmarks/RULES.md:24b`, `.agents/benchmarks/RULES.md:25`).
   - **k8s** — the shared chart `benchmarks/_chart`, selected with
     `--set benchmark=<name>`. A standard benchmark needs nothing here. One with
     bespoke topology adds `benchmarks/_chart/presets/<name>.yaml` to compose its
     sidecars/`Deployment`s/`Service`s through the chart's hooks
     (`initContainers`, `runnerArgs`, `runnerExtraEnv`, `extraManifests`, …) —
     do NOT redeclare the otelcol/gateway/runner Pod
     (`.agents/benchmarks/RULES.md:24b`, `.agents/benchmarks/RULES.md:25`).

   For a simple shared-env benchmark, copy `benchmarks/aime/` and substitute the
   name (no preset needed). Changes to the compose base (`compose/services.yaml`)
   or the chart (`benchmarks/_chart`) MUST be reflected in the other in the same
   commit.

10. **Parameterize the task and enforce limits/isolation in every surface.**
    - Task: shared-env `compose.yaml` MUST read `TASK_ID: ${TASK_ID:-0}` (never
      hardcode a literal); the k8s task comes from `helm --set task=` (default
      0), so a preset MUST NOT hardcode a task. Per-task benchmarks bake
      `EVAL_TASK_ID` via build `ARG` and the artifacts inherit it
      (`.agents/benchmarks/RULES.md:24c`).
    - Resource limits: declare CPU and memory in BOTH `compose.yaml`
      (`deploy.resources.limits` on the runner) and the k8s runner — the chart
      default, overridden per benchmark via its preset's `resources:` —
      matching modulo k8s unit syntax
      (`.agents/benchmarks/RULES.md:10`, `.agents/benchmarks/RULES.md:24e`).
    - Network: enforce no-agent-internet per surface — `internal: true` in
      compose, `iptables --uid-owner` in single mode, credential isolation
      (rule 8) in k8s. If the benchmark *requires* internet, declare
      `eval.benchmark.internet=true` and remove the isolation in every surface;
      asymmetry is forbidden (`.agents/benchmarks/RULES.md:24d`).

11. **Add the tests, then earn the release label.** Provide a build test
    (Dockerfile builds + correct `eval.*` labels,
    `.agents/benchmarks/RULES.md:26`), a compose-config test
    (`.agents/benchmarks/RULES.md:27`), and at least one end-to-end replay test
    with a recorded fixture that verifies `result.json` schema
    (`.agents/benchmarks/RULES.md:28`). The triple-mode CI gate
    (`.agents/benchmarks/RULES.md:29`) checks the per-benchmark files exist and
    parse, the chart renders for the benchmark, and all surfaces share one env
    contract. Once the benchmark is proven end-to-end against
    at least one agent with a replay fixture at
    `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`, add
    `LABEL eval.benchmark.released="true"` (`.agents/benchmarks/RULES.md:21a`).

## Rules served

- `.agents/benchmarks/RULES.md:1-29` — the
  benchmark contract this skill produces (self-contained, version knobs,
  isolation, three-phase execution, task format, scoring, labels, the three
  deployment surfaces, and testing).

## References

- [`assets/TEMPLATE.md`](assets/TEMPLATE.md) — copyable scaffold with the
  Dockerfile, the per-benchmark deploy files, the blanks-to-fill table, and gotchas.
- `.agents/benchmarks/RULES.md` — the outcomes every benchmark MUST satisfy.
- `benchmarks/aime/` — canonical simple shared-env reference.
- `benchmarks/terminal-bench/` — canonical per-task built-from-source reference
  (`build.sh` + `FROM ${TASK_BASE}` overlay + fetch-the-gold `solution.sh`).
- `benchmarks/_chart/` — the shared k8s Helm chart (the canonical Pod, once).
