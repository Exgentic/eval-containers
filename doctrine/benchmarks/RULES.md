# Benchmarks

**Status:** Active
**Date:** April 2026

## Abstract

A benchmark image contains everything needed to evaluate an agent on a task. This document defines the requirements for building benchmark images in Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Self-Contained

1. **Standalone.** The image MUST contain all task data, test logic, and entrypoints, and `docker run` or `docker compose up` MUST work without Dock installed and without internet; `EVAL_TASK_ID` is the only required runtime input for shared-env benchmarks and a build-time argument for per-task benchmarks.

2. **Single input.** The image MUST resolve the task content, expected answer, and attached files from `EVAL_TASK_ID` alone.

3. **Reproducible by default.** The exact dataset version MUST be pinned at build time as a default in the Dockerfile and recorded in `eval.benchmark.data_revision`, and the image MUST produce identical task content on every build when no env vars are set.

4. **Runtime version override.** The entrypoint MUST read `EVAL_BENCHMARK_VERSION` and, when set, materialize that dataset revision into `/tasks/` in place of the default and write the resolved revision to `/output/task/version.json` before the agent runs.

### Isolation

5. **Least privilege.** The agent MUST have access only to what the task needs, and evaluation code, grading logic, expected answers, rubrics, and test infrastructure MUST be inaccessible to it.

6. **Simplest isolation.** The simplest mechanism that achieves the required isolation MUST be used.

7. **Minimal agent environment.** Only `TASK`, `EVAL_TASK_ID`, `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent process, and benchmark internals MUST NOT leak.

8. **No agent access to credentials.** The agent process MUST NOT be able to read LLM credentials.

9. **No agent internet by default.** The agent MUST NOT have outbound internet access unless the benchmark explicitly requires it.

10. **Resource limits.** Every benchmark MUST specify CPU and memory limits in its compose file.

11. **Docker-native security.** Isolation MUST use standard Docker features only, and Dock MUST NOT invent security abstractions.

### Execution

12. **Three-phase flow.** Execution MUST follow agent runs, then test runs, then result is written, via the shared `dock-entrypoint.sh`, which benchmarks MUST NOT bypass.

13. **Agent as non-root.** The agent MUST run as an unprivileged user; the test phase MAY run as root.

14. **Timeout.** Agent execution MUST be bounded by `EVAL_TIMEOUT`.

### Task Format

15. **Stable task IDs.** Shared-env benchmark tasks MUST be addressable by sequential integers with the original upstream identifier stored in `id.txt`; per-task benchmarks use the upstream identifier as the task ID and SHOULD publish a `tasks.txt` mapping.

16. **Flat files.** Task data SHOULD be stored as plain files, with no databases or archive formats.

17. **Agent-visible files.** Attached files the agent needs MUST be placed in a location the agent can read, and the benchmark MUST NOT give the agent read access to the full task store.

### Scoring

18. **Reward contract.** The test script MUST write a reward to `/logs/verifier/reward.txt` as a float in `[0.0, 1.0]`, or `-1` for externally graded benchmarks.

19. **Simplest correct scorer.** Benchmarks SHOULD use the simplest scoring method that produces correct results.

20. **External grading.** A benchmark whose scoring requires an outside service MUST still collect the agent's output, MUST write `-1` as the reward, and MUST NOT approximate the external grader.

### Image

21. **Labels.** Every benchmark image MUST include labels `eval.type`, `eval.benchmark.name`, `eval.benchmark.description`, `eval.benchmark.tasks`, `eval.benchmark.env`, and `eval.benchmark.internet`, and released benchmarks MUST also carry `eval.benchmark.released="true"` (see principle 21a).

21a. **Release readiness gate.** A benchmark is released once proven end-to-end against at least one agent with a recorded replay fixture under `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`, and only then MUST it carry `LABEL eval.benchmark.released="true"`; `doctrine/verification/audit-fleet/references/checklist.md` question 3 checks this label, not the directory count.

21b. **Upstream base tracking.** A benchmark whose `FROM` points at a third-party registry not under Dock's control MUST declare `LABEL eval.benchmark.upstream_base="<full image ref>"` recording the exact upstream reference; `doctrine/verification/audit-fleet/references/checklist.md` question 6 walks every `upstream_base` label and reports yellow if any still points at `:latest`.

22. **Shared components.** Benchmarks SHOULD use shared core images when applicable and MUST NOT reimplement shared logic.

23. **No agent tooling.** Benchmark images MUST NOT include agent-specific tools; the agent's `install.sh` installs what it needs.

### Three Deployment Surfaces

24. **Triple-mode contract — every benchmark ships exactly three deployment artifacts**, one per surface. The artifacts MUST share the same env contract (`EVAL_MODEL`, `EVAL_TASK_ID`, upstream credentials), MUST produce byte-equivalent `task/result.json` outputs for the same inputs (modulo timestamps and provider-side request IDs), and MUST exercise the same five-unit pipeline (otelcol → gateway → agent → verifier → result).

    | File | Mode | Topology | Invocation |
    |------|------|----------|------------|
    | `container.Dockerfile` | **single** | 1 container, 5 processes inside (process-compose orchestrates) | `docker build -f benchmarks/<x>/container.Dockerfile -t <tag> . && docker run <tag>` |
    | `compose.yaml` | **compose** | 3 services on a compose network (otelcol + gateway + runner) | `docker compose -f benchmarks/<x>/compose.yaml up` |
    | `values.yaml` | **k8s** | A Helm values file over the shared chart `benchmarks/_chart` — renders 1 `Job` + 1 Pod + 3 containers (`shareProcessNamespace` for sidecar reaping; isolation via credentials — see 24d), plus any bespoke `Deployment`s/`Service`s the benchmark composes | `helm template benchmarks/_chart -f benchmarks/<x>/values.yaml \| kubectl apply -f -` |

    A benchmark that ships only one or two surfaces is incomplete.

24a. **Universal eval-image recipe.** `core/combination.Dockerfile` is the single source of truth for the eval-image build; each per-benchmark `container.Dockerfile` MUST be a single-line `FROM <registry>/evals/<benchmark>--<agent>:<tag>`, the canonical build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`, `MODEL_IMAGE`) MUST be recorded in the benchmark's `README.md`, inert `ARG` lines the `FROM` does not consume are forbidden, and duplicating the combination Dockerfile body is forbidden.

24b. **Both surfaces share one base; benchmarks override only what differs.** `compose/services.yaml` holds the shared compose services, pulled into each per-benchmark `compose.yaml` via `include:`; `benchmarks/_chart/` defines the shared Helm Job once, and each per-benchmark `values.yaml` pins the benchmark and overrides only what differs through the chart's composition hooks. A change to the compose base or the chart MUST be reflected in the other in the same commit.

24c. **Task parameterization in deployment artifacts.** Shared-env `compose.yaml` MUST read the task id as `TASK_ID: ${TASK_ID:-0}` and MUST NOT hardcode it, the k8s surface MUST parameterize the task via `helm … --set task=` and `values.yaml` MUST NOT hardcode it, and per-task benchmarks bake `EVAL_TASK_ID` as a build-time `ARG`.

24d. **Network isolation across surfaces.** Rule 9 MUST be enforced in every shipped surface by its native mechanism: `internal: true` on the agent's network in `compose.yaml`, `iptables -m owner --uid-owner` rules in single mode, and credential isolation (rule 8) in k8s; a benchmark requiring agent internet MUST declare `eval.benchmark.internet=true` and remove the isolation primitive in every shipped surface, and asymmetry is forbidden.

24e. **Resource limit parity.** Rule 10's limits MUST be expressed in both `compose.yaml` (`deploy.resources.limits`) and the k8s surface (`resources.limits`), and the values MUST match modulo k8s unit syntax.

25. **Use the surface's natural sharing approach.** Per-benchmark `compose.yaml` MUST pull `compose/services.yaml` in via `include:` and only declare overrides, with inlining a definition that already exists there forbidden; per-benchmark `values.yaml` MUST only pin the benchmark and override what differs, with re-declaring the shared Pod forbidden.

### Testing

26. **Build test.** Every benchmark image MUST have a build test verifying the Dockerfile builds and produces correct `dock.*` labels.

27. **Compose test.** Every benchmark MUST have a compose validation test verifying `docker compose config` succeeds.

28. **Replay test.** Every benchmark MUST have at least one end-to-end test using the replay model with a recorded fixture, verifying `result.json` is produced with the correct schema.

29. **Triple-mode existence + render test.** A CI test MUST walk every directory in `benchmarks/` and assert:
    (a) `container.Dockerfile`, `compose.yaml`, and `values.yaml` all exist;
    (b) `container.Dockerfile` is a single-line `FROM` (rule 24a);
    (c) `docker compose -f compose.yaml config` succeeds (rule 27);
    (d) `helm template benchmarks/_chart -f benchmarks/<x>/values.yaml` renders and validates against the k8s schema;
    (e) `values.yaml` pins the benchmark, so the chart renders `evals/<name>--<agent>` and labels the Job from it;
    (f) the env contract is identical across all three surfaces.
    A benchmark failing any sub-test MUST NOT be merged.

## References

- [Process](../RULES.md)
- [Agents](../agents/RULES.md)
- [Repository](../compose/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 3 into rule 3 (reproducible by default via pinned `ARG DATA_REVISION`) and new rule 4 (runtime override via `EVAL_BENCHMARK_VERSION`, writes resolved revision to `/output/task/version.json`). Renumbered rules 5–28. Standardized `TASK_ID` → `EVAL_TASK_ID` in the minimal-agent-environment rule. |
| 2026-04-30 | Reframed rules 1, 8, 9, 24 to support dual artifact shapes (single-image + compose). Rule 1 now accepts `docker run` OR `docker compose up`. Rules 8 and 9 reframe credential and network isolation in terms of the agent process (achievable via separate container OR UID separation). Rule 24 keeps compose as source of truth and adds the per-(benchmark, agent) Dockerfile as the deployable artifact for collapsible benchmarks. |
| 2026-05-18 | Rule 24 rewritten as the triple-mode contract: every benchmark ships `container.Dockerfile` (single) + `compose.yaml` (compose) + `job.yaml` (k8s). Rule 24a forbids duplicating the universal `core/combination.Dockerfile` body per benchmark — per-benchmark Dockerfiles only pin build args. Rule 24b requires `compose.yaml` and `job.yaml` to stay in lockstep. Pre-rename `single.yaml` is gone (it was a one-container k8s adapter for single mode — but single mode's contract is the Dockerfile, not a YAML); `k8s.yaml` renamed to `job.yaml`. |
| 2026-05-18 | Tightening pass before the 90-benchmark sweep. Rule 24a corrected: `container.Dockerfile` MUST be a single-line registry pin; inert `ARG` lines that the `FROM` doesn't consume are forbidden (they looked load-bearing but drifted). New rule 24c codifies task parameterization — shared-env `compose.yaml` MUST use `${TASK_ID:-0}`, `job.yaml` ships as a task-0 template; per-task benchmarks bake `EVAL_TASK_ID` via build ARG. New rule 24d makes network-isolation enforcement explicit per surface and honest about k8s achieving rule 9 via credential isolation (rule 8) rather than network policy. New rule 24e requires resource limits to be declared identically in both `compose.yaml` and `job.yaml`. Rule 25 strengthened to forbid inlining definitions that already exist in `compose/services.yaml`. New rule 29 mandates a triple-mode CI gate that walks `benchmarks/` and asserts artifact existence + parse + env-contract symmetry. |
| 2026-06-01 | k8s surface moved from per-benchmark Kustomize to one shared **Helm chart** (`benchmarks/_chart`) + a per-benchmark `values.yaml`. Rule 24's k8s artifact is `values.yaml` (was `job.yaml`); 24b/25 replace the `benchmarks/_base/job.yaml` inline-and-drift model with "the Pod is defined once in the chart; `values.yaml` pins the benchmark and overrides only what differs"; 24c parameterizes the task via `helm --set task=`; 24e/24d retargeted at the chart. Rule 29 drops the canonical-drift sub-test (one chart can't drift) — it now renders each `values.yaml` via `helm template` and kubeconform-validates. `eval-containers run --mode job` and `--overlay` drive Helm; the OpenShift overlay is `deploy/values-openshift.yaml`. Deleted `benchmarks/_base` + 114 per-benchmark kustomize files (net −5.4k lines). |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
