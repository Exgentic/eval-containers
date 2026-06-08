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
benchmark that satisfies `doctrine/benchmarks/RULES.md`. Read that RULES.md
before starting. A copyable starting point lives at
[`assets/TEMPLATE.md`](assets/TEMPLATE.md) — the steps below tell you how to use
it and why each piece exists.

## Before you start: pick the benchmark shape

Decide between the two task models, because it changes how `EVAL_TASK_ID` flows:

- **Shared-env** — one image, many tasks; all tasks share the same environment
  and only the instruction differs (AIME, SimpleQA, GPQA). `EVAL_TASK_ID` is the
  *only required runtime input* and selects the task at run time.
- **Per-task** — one image per task; `EVAL_TASK_ID` is a *build-time* `ARG` and
  each image bakes exactly one task (SWE-bench style; see
  `benchmarks/swe-bench/Dockerfile`).

Either `docker run` (single-image) or `docker compose up` (multi-service) MUST
work with no Dock install and no internet, resolving task content, expected
answer, and any attached files from `EVAL_TASK_ID` alone
(`doctrine/benchmarks/RULES.md:1`, `doctrine/benchmarks/RULES.md:2`).

## Steps

1. **Create the benchmark directory.** Make `benchmarks/<name>/` and copy
   [`assets/TEMPLATE.md`](assets/TEMPLATE.md) as your scaffold. Every benchmark
   ships four authored files: `Dockerfile` (builds the base image with
   tasks + verifier), the per-benchmark deploy files `container.Dockerfile` +
   `compose.yaml`, and `README.md`. The k8s surface is the shared chart selected
   with `--set benchmark=<name>` — author a `benchmarks/_chart/presets/<name>.yaml`
   only if the benchmark needs bespoke topology. *Why:* the directory is the
   unit a CI test walks; missing `container.Dockerfile` or `compose.yaml` makes
   the benchmark incomplete (`doctrine/benchmarks/RULES.md:24`, `doctrine/benchmarks/RULES.md:29`).

2. **Write the `Dockerfile` to materialize tasks as flat files.** Fetch the
   dataset and write each task as `/tasks/<id>/problem.txt` + `answer.txt`
   (+ `id.txt`), then `chmod -R 600 /tasks` so the agent UID cannot read them.
   *Why:* tasks SHOULD be plain files, no databases or archives
   (`doctrine/benchmarks/RULES.md:16`); shared-env tasks MUST be addressable by
   sequential integers with the upstream id preserved in `id.txt`
   (`doctrine/benchmarks/RULES.md:15`); and the agent MUST NOT be able to read
   anything the test phase uses (`doctrine/benchmarks/RULES.md:5`,
   `doctrine/benchmarks/RULES.md:28`). If a task has attached files, copy them
   into `/app/` (agent-readable) in the entrypoint — never loosen `/tasks/`
   permissions (`doctrine/benchmarks/RULES.md:17`).

3. **Pin the dataset version (knob 1 of 2 — build-time default).** Set
   `ARG DATA_REVISION=<sha>` (or equivalent) as the default and record it in the
   `eval.benchmark.data_revision` label. *Why:* the image MUST produce identical
   task content on every build when no env vars are set
   (`doctrine/benchmarks/RULES.md:3`). Get the sha with
   `curl -s https://huggingface.co/api/datasets/<dataset> | jq .sha`.

4. **Honor the runtime version override (knob 2 of 2 — `EVAL_BENCHMARK_VERSION`).**
   The entrypoint MUST read `EVAL_BENCHMARK_VERSION` and, when set, fetch and
   materialize that dataset revision into `/tasks/` in place of the default,
   writing the resolved revision to `/output/task/version.json` before the agent
   runs. When unset, the build-time default applies unchanged. *Why:* this is
   how a caller evaluates against a specific dataset revision without rebuilding
   (`doctrine/benchmarks/RULES.md:4`). Note `EVAL_BENCHMARK_TAG` selects the
   image tag to pull — that is Docker's job, not the entrypoint's.

5. **Reuse the shared entrypoint and verifier; do not reimplement them.** `COPY`
   in `core/entrypoint`'s `dock-entrypoint.sh` and, for exact-match scoring,
   `core/test-exact-match`. Your thin `/entrypoint.sh` only assembles `TASK`
   and `EXPECTED_ANSWER` from `/tasks/$TASK_ID/`, then `exec`s the shared
   entrypoint. *Why:* execution MUST follow the three-phase flow agent → test →
   result via the shared entrypoint, and benchmarks MUST NOT bypass it or
   reimplement shared logic (`doctrine/benchmarks/RULES.md:12`,
   `doctrine/benchmarks/RULES.md:22`). Do NOT bake agent-specific tools
   (browsers, SDKs) into the benchmark image — the agent's `install.sh` brings
   its own (`doctrine/benchmarks/RULES.md:23`).

6. **Keep the agent unprivileged, credential-free, and offline by default.** The
   agent process runs as a non-root user (the test phase MAY be root), cannot
   read the LLM API key, and has no outbound internet unless the benchmark
   explicitly needs it. *Why:* `doctrine/benchmarks/RULES.md:13` (agent as
   non-root), `doctrine/benchmarks/RULES.md:8` (no agent access to credentials),
   `doctrine/benchmarks/RULES.md:9` (no agent internet by default). Use the
   simplest isolation that works — file permissions over extra containers
   (`doctrine/benchmarks/RULES.md:6`), and standard Docker features only
   (`doctrine/benchmarks/RULES.md:11`). Only `TASK`, `EVAL_TASK_ID`,
   `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent; benchmark
   internals MUST NOT leak (`doctrine/benchmarks/RULES.md:7`). Agent execution
   MUST be bounded by `EVAL_TIMEOUT`, which the shared entrypoint enforces — the
   benchmark does not implement its own timeout (`doctrine/benchmarks/RULES.md:14`).

7. **Wire the scorer to the reward contract.** The test script MUST write a
   float in `[0.0, 1.0]` to `/logs/verifier/reward.txt` (or `-1` for externally
   graded benchmarks). Use the simplest correct scorer — exact match when
   possible, code execution for programming, LLM-as-judge only when nothing
   simpler works. *Why:* `doctrine/benchmarks/RULES.md:18` (reward contract),
   `doctrine/benchmarks/RULES.md:19` (simplest correct scorer). If grading needs
   an outside service, still collect the agent's output, write `-1`, and do NOT
   approximate the external grader (`doctrine/benchmarks/RULES.md:20`). For a
   custom scorer, replace the `test-exact-match` COPY with your own
   `/grade.sh`.

8. **Set the required labels.** Every benchmark image MUST carry `eval.type`,
   `eval.benchmark.name`, `eval.benchmark.description`, `eval.benchmark.tasks`,
   `eval.benchmark.env`, and `eval.benchmark.internet`
   (`doctrine/benchmarks/RULES.md:21`). If the `FROM` points at a third-party
   registry outside Dock's control, also declare
   `eval.benchmark.upstream_base="<full image ref>"` so the external dependency
   is visible to audit tools (`doctrine/benchmarks/RULES.md:21b`). Do NOT add
   `eval.benchmark.released="true"` yet — that label is earned at the release
   gate (step 11).

9. **Author the three deployment surfaces with one shared env contract.** The
   surfaces MUST share the same env contract (`EVAL_MODEL`, `EVAL_TASK_ID`,
   upstream credentials) and produce byte-equivalent `task/result.json` for the
   same inputs (`doctrine/benchmarks/RULES.md:24`):
   - `container.Dockerfile` (**single**) — a *single-line* registry pin
     `FROM <registry>/evals/<name>--<agent>:<tag>`, nothing more. Record the
     canonical build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`,
     `MODEL_IMAGE`) in the `README.md` so CI can rebuild via
     `core/combination.Dockerfile`. Inert `ARG` lines the `FROM` does not
     consume are forbidden (`doctrine/benchmarks/RULES.md:24a`).
   - `compose.yaml` (**compose**) — pull in `compose/services.yaml` via
     `include:` and only declare overrides; do NOT inline a service, network, or
     volume that already exists there
     (`doctrine/benchmarks/RULES.md:24b`, `doctrine/benchmarks/RULES.md:25`).
   - **k8s** — the shared chart `benchmarks/_chart`, selected with
     `--set benchmark=<name>`. A standard benchmark needs nothing here. One with
     bespoke topology adds `benchmarks/_chart/presets/<name>.yaml` to compose its
     sidecars/`Deployment`s/`Service`s through the chart's hooks
     (`initContainers`, `runnerArgs`, `runnerExtraEnv`, `extraManifests`, …) —
     do NOT redeclare the otelcol/gateway/runner Pod
     (`doctrine/benchmarks/RULES.md:24b`, `doctrine/benchmarks/RULES.md:25`).

   For a simple shared-env benchmark, copy `benchmarks/aime/` and substitute the
   name (no preset needed). Changes to the compose base (`compose/services.yaml`)
   or the chart (`benchmarks/_chart`) MUST be reflected in the other in the same
   commit.

10. **Parameterize the task and enforce limits/isolation in every surface.**
    - Task: shared-env `compose.yaml` MUST read `TASK_ID: ${TASK_ID:-0}` (never
      hardcode a literal); the k8s task comes from `helm --set task=` (default
      0), so a preset MUST NOT hardcode a task. Per-task benchmarks bake
      `EVAL_TASK_ID` via build `ARG` and the artifacts inherit it
      (`doctrine/benchmarks/RULES.md:24c`).
    - Resource limits: declare CPU and memory in BOTH `compose.yaml`
      (`deploy.resources.limits` on the runner) and the k8s runner — the chart
      default, overridden per benchmark via its preset's `resources:` —
      matching modulo k8s unit syntax
      (`doctrine/benchmarks/RULES.md:10`, `doctrine/benchmarks/RULES.md:24e`).
    - Network: enforce no-agent-internet per surface — `internal: true` in
      compose, `iptables --uid-owner` in single mode, credential isolation
      (rule 8) in k8s. If the benchmark *requires* internet, declare
      `eval.benchmark.internet=true` and remove the isolation in every surface;
      asymmetry is forbidden (`doctrine/benchmarks/RULES.md:24d`).

11. **Add the tests, then earn the release label.** Provide a build test
    (Dockerfile builds + correct `eval.*` labels,
    `doctrine/benchmarks/RULES.md:26`), a compose-config test
    (`doctrine/benchmarks/RULES.md:27`), and at least one end-to-end replay test
    with a recorded fixture that verifies `result.json` schema
    (`doctrine/benchmarks/RULES.md:28`). The triple-mode CI gate
    (`doctrine/benchmarks/RULES.md:29`) checks the per-benchmark files exist and
    parse, the chart renders for the benchmark, and all surfaces share one env
    contract. Once the benchmark is proven end-to-end against
    at least one agent with a replay fixture at
    `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`, add
    `LABEL eval.benchmark.released="true"` (`doctrine/benchmarks/RULES.md:21a`).

## Rules served

- `doctrine/benchmarks/RULES.md:1-29` — the
  benchmark contract this skill produces (self-contained, version knobs,
  isolation, three-phase execution, task format, scoring, labels, the three
  deployment surfaces, and testing).

## References

- [`assets/TEMPLATE.md`](assets/TEMPLATE.md) — copyable scaffold with the
  Dockerfile, the per-benchmark deploy files, the blanks-to-fill table, and gotchas.
- `doctrine/benchmarks/RULES.md` — the outcomes every benchmark MUST satisfy.
- `benchmarks/aime/` — canonical simple shared-env reference.
- `benchmarks/_chart/` — the shared k8s Helm chart (the canonical Pod, once).
