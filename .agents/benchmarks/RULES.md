# Benchmarks

**Status:** Active
**Date:** April 2026

## Abstract

A benchmark image contains everything needed to evaluate an agent on a task. This document defines the requirements for building benchmark images in Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Self-Contained

1. **Standalone.** The image MUST contain all task data, test logic, and entrypoints. Either `docker run` (single-image benchmarks) or `docker compose up` (multi-service benchmarks) MUST work without Dock installed and without internet. For shared-env benchmarks, `EVAL_TASK_ID` is the only required runtime input. For per-task benchmarks, `EVAL_TASK_ID` is a build-time argument — each image contains exactly one task.

2. **Single input.** The image MUST resolve to the task content, expected answer, and any attached files from `EVAL_TASK_ID` alone — whether provided at build time (per-task) or runtime (shared-env).

3. **Reproducible by default.** The exact dataset version MUST be pinned at build time as a default in the Dockerfile (`ARG DATA_REVISION=<sha>` or equivalent) and recorded in `eval.benchmark.data_revision`. The image MUST produce identical task content on every build when no env vars are set.

4. **Version is a build arg.** The dataset revision MUST be a single `ARG BENCHMARK_VERSION=<rev>` that drives **both** the fetch/materialize step and the `eval.benchmark.data_revision` label, so the label can never disagree with the data baked in. Override at build (`build bench --benchmark-version <rev>`); unset uses the pinned revision. The revision is immutable per image — there is no runtime override (reproducibility: the data is whatever the image was built with). `EVAL_BENCHMARK_TAG` selects which image tag to pull — that's Docker's job, not the launcher's.

### Isolation

5. **Least privilege.** The agent MUST have access only to what it needs to perform the task — nothing more. Evaluation code, grading logic, expected answers, rubrics, and test infrastructure MUST be inaccessible to the agent. The agent MUST NOT be able to read or modify anything used by the test phase.

6. **Simplest isolation.** Use the simplest mechanism that achieves the required isolation. File permissions over separate containers. User separation over network policies. Complexity is the enemy of security.

7. **Minimal agent environment.** The agent process receives only what it needs to *attempt* the task: `TASK` (the problem text), the gateway endpoints (`OPENAI_BASE_URL`, `ANTHROPIC_BASE_URL`), and operational vars (`MODEL`, `TIMEOUT`). It MUST NOT receive the task **identity** (`EVAL_TASK_ID` / `TASK_ID`) — a model that recognizes a benchmark instance id can recall a memorized solution and inflate the score. Grading code, expected answers, rubrics, and other benchmark internals MUST NOT leak either. The verifier and result steps DO need the task id; they read it from the container env, not from the agent's (the agent runs under `env -i` with an explicit allow-list that excludes it).

8. **No agent access to credentials.** The agent process MUST NOT have access to LLM credentials. In compose-mode benchmarks, credentials live in the separate model service. In single-image benchmarks, credentials live in a 0600 file owned by a proxy UID inaccessible to the agent UID. Both implementations MUST achieve the same property: the agent process cannot read the API key.

9. **No agent internet by default.** The agent MUST NOT have outbound internet access unless the benchmark explicitly requires it. In compose-mode benchmarks, this is enforced by `internal: true` on the agent's network. In single-image benchmarks, by `iptables -m owner --uid-owner` rules on the agent UID. Both implementations MUST achieve the same property: the agent has no path to the open internet by default.

10. **Resource limits.** Every benchmark MUST specify CPU and memory limits in its compose file.

11. **Docker-native security.** Isolation MUST use standard Docker features only (networks, capabilities, read-only filesystem, tmpfs, resource limits). Dock MUST NOT invent security abstractions. If Docker can't enforce it, Dock doesn't promise it.

### Execution

12. **Three-phase flow.** Execution MUST follow: agent runs → test runs → result is written. The process-compose pipeline (orchestrated by `/usr/local/bin/run`) handles this. Benchmarks MUST NOT bypass it.

13. **Agent as non-root.** The agent MUST run as an unprivileged user. The test phase MAY run as root.

14. **Timeout.** Agent execution MUST be bounded by `EVAL_TIMEOUT`. The framework launcher (`/usr/local/bin/run`) bridges this to `$TIMEOUT`; process-compose's agent command enforces it via `timeout $TIMEOUT`.

### Task Format

15. **Stable task IDs.** For shared-env benchmarks, tasks MUST be addressable by sequential integers (`0`, `1`, `2`, ...) with the original upstream identifier stored in `id.txt`. For per-task benchmarks, the upstream identifier is the task ID. Per-task benchmarks SHOULD publish a `tasks.txt` file (one ID per line) so integers can be mapped to original IDs.

16. **Flat files.** Task data SHOULD be stored as plain files (`problem.txt`, `answer.txt`). No databases, no archive formats.

17. **Agent-visible files.** If the agent needs attached files (documents, images), they MUST be placed in a location the agent can read (e.g., `/app/`). The benchmark MUST NOT give the agent read access to the full task store.

### Scoring

18. **Reward contract.** The test script MUST write a reward to `/logs/verifier/reward.txt`. The value MUST be a float in `[0.0, 1.0]`, or `-1` for externally graded benchmarks.

19. **Simplest correct scorer.** Benchmarks SHOULD use the simplest scoring method that produces correct results. Exact match when possible, code execution for programming, LLM-as-judge only when nothing simpler works.

20. **External grading.** If scoring requires an outside service, the benchmark MUST still collect the agent's output. It MUST write `-1` as the reward. It MUST NOT approximate the external grader.

20a. **Oracle derives the gold.** The oracle's gold solution MUST derive the expected output from the task's own inputs rather than embed or copy a precomputed answer.

### Image

21. **Labels.** Every benchmark image MUST include labels: `eval.type`, `eval.benchmark.name`, `eval.benchmark.description`, `eval.benchmark.tasks`, `eval.benchmark.env`, `eval.benchmark.internet`, and `eval.benchmark.data_revision` (rule 3). Benchmarks that have graduated to the release gate MUST also carry `eval.benchmark.released="true"` (see principle 21a).

21a. **Release readiness gate.** A benchmark is **released** when it has been proven end-to-end against at least one agent with a recorded replay fixture under `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`. Released benchmarks MUST carry `LABEL eval.benchmark.released="true"`. Unreleased benchmarks MAY exist in `benchmarks/` (the source tree is the full catalog of what Dock COULD support), but MUST NOT carry the label until a fixture lands and they pass the replay sweep. `.agents/verification/audit-fleet/references/checklist.md` question 3 (replay coverage) checks this label, not the directory count — the filesystem can hold 96 benchmarks while only a subset are released.

21b. **Upstream base tracking.** Benchmarks whose `FROM` line points at a third-party registry not under Dock's control (e.g. `ghcr.io/andyzorigin/*`, `ghcr.io/openai/*`) MUST declare `LABEL eval.benchmark.upstream_base="<full image ref>"` recording the exact upstream reference. This makes the external dependency visible to audit tools and to anyone reading the image metadata. Benchmarks that build from a Dock-controlled or fully in-repo base (e.g. `FROM python:3.12-slim`) do NOT need this label. `.agents/verification/audit-fleet/references/checklist.md` question 6 (stale upstream images) walks every `upstream_base` label and reports yellow if any still points at `:latest` — such bases are legal but flagged as known supply-chain debt until mirrored or pinned by digest.

22. **Shared components.** Benchmarks SHOULD use shared core images (`/usr/local/bin/run`, `test-exact-match`) when applicable. Benchmarks MUST NOT reimplement shared logic.

23. **No agent tooling.** Benchmark images MUST NOT include agent-specific tools (browsers, automation libraries, SDKs). The agent's `install.sh` installs what it needs. The benchmark provides the environment, not the tools.

### Three Deployment Surfaces

24. **Triple-mode contract — every benchmark supports three deployment surfaces.** The single and compose surfaces each ship a per-benchmark file; the k8s surface is the shared chart `benchmarks/_chart`, selected by name (`--set benchmark=<x>`), with an optional per-benchmark preset for bespoke topology. All three MUST share the same env contract (`EVAL_MODEL`, `EVAL_TASK_ID`, upstream credentials), MUST produce byte-equivalent `task/result.json` outputs for the same inputs (modulo timestamps and provider-side request IDs), and MUST exercise the same five-unit pipeline (otelcol → gateway → agent → verifier → result).

    | Artifact | Mode | Topology | Invocation |
    |------|------|----------|------------|
    | `container.Dockerfile` | **single** | 1 container, 5 processes inside (process-compose orchestrates) | `docker build -f benchmarks/<x>/container.Dockerfile -t <tag> . && docker run <tag>` |
    | `compose.yaml` | **compose** | 3 services on a compose network (otelcol + gateway + runner) | `docker compose -f benchmarks/<x>/compose.yaml up` |
    | shared chart + `presets/<x>.yaml` *(optional)* | **k8s** | The shared chart `benchmarks/_chart` renders 1 `Job` + 1 Pod + 3 containers (`shareProcessNamespace` for sidecar reaping; isolation via credentials — see 24d), plus any bespoke `Deployment`s/`Service`s the benchmark composes via its preset | `helm template benchmarks/_chart --set benchmark=<x> \| kubectl apply -f -` |

    Single mode is the simplest surface (one `docker run`, no orchestrator); compose and k8s split the pipeline across containers so the agent process cannot see upstream credentials (rule 8). Benchmarks missing the single or compose file are incomplete; the k8s surface works for every benchmark with no per-benchmark file (a preset is added only for bespoke topology).

24a. **Universal eval-image recipe.** `core/combination.Dockerfile` is the single source of truth for the eval-image build. Per-benchmark `container.Dockerfile` files MUST be a single-line registry pin of the form `FROM <registry>/evals/<benchmark>--<agent>:<tag>` — nothing more. **Per-task benchmarks** (rule 24f) are the sole exception: because their eval image is one-per-task, their `container.Dockerfile` is exactly two lines — `ARG EVAL_TASK_ID` followed by `FROM <registry>/evals/<benchmark>-${EVAL_TASK_ID}--<agent>:<tag>` (the `ARG` is consumed by the `FROM`, so it is load-bearing, not inert). The canonical build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`, `MODEL_IMAGE`) MUST be recorded in the benchmark's `README.md` so CI can rebuild the eval image by invoking `core/combination.Dockerfile` with those args. Declaring inert `ARG` lines that the `FROM` does not consume is forbidden — they look load-bearing but aren't, and they drift. Duplicating the combination Dockerfile body across benchmarks is forbidden — there is exactly one eval-image recipe in the repo.

24b. **Both surfaces share one base; benchmarks override only what differs.**
    - `compose/services.yaml` — shared compose services (`otelcol`, `gateway`, `runner`); per-benchmark `compose.yaml` pulls it in via `include:` and overrides the benchmark-specific bits.
    - `benchmarks/_chart/` — the shared Helm chart: the otelcol + gateway + runner Job, defined **once**. The benchmark is named via `--set benchmark=<x>`; standard benchmarks override nothing. A benchmark with bespoke topology adds `benchmarks/_chart/presets/<name>.yaml` — bundled in the chart and overlaid automatically when selected — to compose its sidecars/`Deployment`s/`Service`s through the chart's hooks (`initContainers`, `runnerArgs`, `runnerExtraEnv`, `extraManifests`, …). Presets MUST set only structural keys, never the per-run axes (agent/task/model), which arrive via `--set`.

    Both surfaces keep `service_healthy` ↔ `shareProcessNamespace` reaper, service-name DNS ↔ Pod loopback, and an identical env contract in lockstep — changes to the compose base or the chart MUST be reflected in the other in the same commit.

24c. **Task parameterization in deployment artifacts.** Rule 1 makes `EVAL_TASK_ID` the only required runtime input for shared-env benchmarks; the deployment artifacts MUST honor this:
    - **Shared-env**: `compose.yaml` MUST read the task id from the shell environment as `TASK_ID: ${TASK_ID:-0}` (default 0, override via `TASK_ID=42 docker compose up`). Hardcoding a literal task id in `compose.yaml` is forbidden. The k8s surface parameterizes the task through Helm — `helm template … --set task=42` (default 0); a benchmark's preset MUST NOT hardcode a task id.
    - **Per-task**: each task bakes a **separate eval image** named `evals/<benchmark>-<task>--<agent>` (rule 24f). `container.Dockerfile` takes `EVAL_TASK_ID` as a build `ARG` (pinning one task's image); `compose.yaml` and the chart select that per-task image **by name** from `EVAL_TASK_ID` / `--set task`. Per-task benchmarks run **one Job per task** — they MUST NOT use the Indexed dataset Job (one image × N indices is valid only for shared-env benchmarks, where the image is fixed and the task is the index); the chart rejects `datasetSize` when the benchmark is per-task.

24d. **Network isolation across surfaces.** Rule 9 (no agent internet by default) MUST be enforced in every shipped surface, by the mechanism native to that surface:
    - `compose.yaml`: agent's network is `internal: true`; the gateway is the only service joined to a separate `upstream` network.
    - `container.Dockerfile` (single mode): `iptables -m owner --uid-owner` rules on the agent UID, applied at container start.
    - **k8s** (the rendered chart): containers in a Pod share the network stack, so a per-container egress firewall is not possible; rule 9 is achieved indirectly via rule 8 (the runner container has no API credentials, so even if it reached `api.openai.com` directly the call would fail auth). A `NetworkPolicy` MAY be added for defense in depth.

    Benchmarks that explicitly require agent internet MUST declare `eval.benchmark.internet=true` AND remove the relevant isolation primitive in every surface that ships. Asymmetry (e.g., compose blocks but k8s allows) is forbidden.

24e. **Resource limit parity.** Rule 10 (resource limits) MUST be expressed in both `compose.yaml` (`deploy.resources.limits` on the runner) and the k8s surface (`resources.limits` on the runner — the chart default, which a benchmark overrides via its preset's `resources:`). The values MUST match modulo k8s unit syntax (`"2"` ↔ `2`, `"2Gi"` ↔ `2147483648`). GPU benchmarks declare via `deploy.resources.reservations.devices[].driver: nvidia` in compose and `resources.limits["nvidia.com/gpu"]` in k8s.

24f. **Eval-image naming.** The combined eval image is named by axis **and environment**, and every surface (build, compose, container, job) MUST address it identically:
    - **shared-env** → `evals/<benchmark>--<agent>` (one image; the task is a runtime input).
    - **per-task** → `evals/<benchmark>-<task>--<agent>` (one image per task; the task id is part of the name, mirroring the per-task base `benchmarks/<benchmark>-<task>`).

    The `--` before `<agent>` is load-bearing (the OpenShift flattener collapses the nested path to a single-segment imagestream name). `eval-containers build eval --task-id X` MUST emit the exact name that `run --task-id X` — and the rendered Job / compose runner — consume; a per-task `build` that drops the task id is a defect. A benchmark is per-task iff its Dockerfile declares `LABEL eval.benchmark.env="per-task"`; the CLI reads that label (`benchmark::is_per_task`) to pick the naming and to set the chart's `perTask` value (the `eval.benchmark.env="per-task"` LABEL MUST stand on its own line — detection matches a `LABEL …env="per-task"` line, not a label folded into a multi-line `LABEL`; the `per_task_label_matches_structure` test enforces label↔structure agreement). Job mode additionally derives a Helm **release name** `<benchmark>-<agent>-task-<task>`; release names are DNS-1123 labels (no `_`) while per-task task ids carry `_` (SWE-bench's `sympy__sympy-24066`), so the CLI sanitizes it (`naming::release_name`) — an unsanitized per-task release name makes `--mode job` unrenderable.

24g. **Per-task environments built from source.** Most per-task benchmarks pin a per-task upstream base image (swe-bench → Epoch's `swe-bench.eval.<arch>.<task>`), so a single `Dockerfile` + `--build-arg EVAL_TASK_ID` builds the image. When **no** such image exists and each task's environment must be built from its own upstream Dockerfile (heterogeneous bases + setup — e.g. terminal-bench), the benchmark instead ships an executable **`build.sh <image> <task-id>`**: it (1) builds the task's own upstream Dockerfile into a task-env image, then (2) overlays the eval pipeline via the benchmark's `Dockerfile` (`FROM ${TASK_BASE}` — instruction, **root-only** tests, grader, entrypoint). `eval-containers build`/`oracle`/`run` invoke `benchmarks/<name>/build.sh` when present instead of the default `docker build` (src/build.rs). The gold solution MUST NOT be baked into the image (rule 9); the oracle's `solution.sh` fetches it fresh at run time.

24h. **Per-task service sidecars.** A benchmark whose tasks each touch a *subset* of a shared set of service sidecars (e.g. webarena's websites) selects that subset per task **without the CLI** — rule 1 (standalone) means `helm`/`docker compose` resolve it from the benchmark's own data, not from `eval-containers`. The benchmark declares a `sidecars:` catalog (name → `{image, port, [shmSize]}`) in its chart preset, plus a committed task→sites map at `<chart>/task-profiles/<benchmark>.json` (generated from the pinned dataset — derived, never hand-maintained). Keys MUST be the same identifier the runtime uses for `EVAL_TASK_ID`: for shared-env benchmarks that means the sequential integer line index per rule 15 (NOT the upstream task identifier, which lives in `id.txt`). The generator emits DNS-1123 service names, so catalog, map, chart, and compose share one naming convention.
    - **k8s/job — the chart self-resolves.** It reads the map (`Files.Get` + `fromJson`), indexes it by the task id, and renders a `Deployment`+`Service` and a readiness gate **only** for the active sidecars. `helm template --set benchmark=<x> --set task=<id>` selects them — no CLI, no `--set activeProfiles`.
    - **compose — selects from the same map.** Bare `EVAL_TASK_ID=<id> docker compose up` brings up the **full** site set (the zero-knowledge standalone default, rule 1, no CLI). compose has no templating, so per-task selection runs in the shell off the *same* map — name the task's sites, e.g. `docker compose up runner proxy $(jq -r --arg t "$EVAL_TASK_ID" '.[$t][]' <chart>/task-profiles/<benchmark>.json)`. The runner depends only on the proxy, so a subset doesn't pull in the rest. One map drives both surfaces.
    Always-on services (e.g. the HAR-capture proxy) aren't task-gated — they live in `extraManifests`. A task MUST start in, and be graded only against, its declared sites, so the subset is faithful. The catalog + map are the single source of truth; the CLI is never required and MUST NOT be the only path to per-task selection. Benchmarks without a catalog/map are unaffected.

24i. **Per-task task identity is baked.** A per-task benchmark's gold solution MUST resolve its task id from a value baked into the image at build time, not from the runtime `EVAL_TASK_ID`.

25. **Use the surface's natural sharing approach.**
    - **compose** has native sharing (`include:`/`extends:`). Per-benchmark `compose.yaml` MUST pull `compose/services.yaml` in via `include:` and only declare overrides. Inlining a service/healthcheck/network/volume that already exists in `compose/services.yaml` is forbidden.
    - **k8s** uses Helm. The shared Pod lives once in the chart's template (`benchmarks/_chart/templates/`); a benchmark is selected with `--set benchmark=<x>` and overrides what differs only via an optional `presets/<x>.yaml` (the chart's composition hooks). Re-declaring the otelcol/gateway/runner Pod in a preset is forbidden — there is exactly one k8s Pod definition in the repo.

    Effect: when the canonical Pod shape evolves, compose changes 1 file and k8s changes 1 file (the chart) — every benchmark re-renders from it, so there is nothing to drift.

### Testing

26. **Build test.** Every benchmark image MUST have a build test that verifies the Dockerfile builds and produces correct `dock.*` labels.

27. **Compose test.** Every benchmark MUST have a compose validation test that verifies `docker compose config` succeeds.

28. **Replay test.** Every benchmark MUST have at least one end-to-end test using the replay model with a recorded fixture. This test MUST verify that `result.json` is produced with the correct schema.

29. **Triple-mode existence + render test.** A CI test MUST walk every directory in `benchmarks/` and assert:
    (a) `container.Dockerfile` and `compose.yaml` exist;
    (b) `container.Dockerfile` is a single-line `FROM` (rule 24a);
    (c) `docker compose -f compose.yaml config` succeeds (rule 27);
    (d) `helm template benchmarks/_chart --set benchmark=<x>` renders (overlaying `presets/<x>.yaml` if present) and its output validates against the k8s schema (kubeconform or equivalent), labelling the Job `evals/<name>--<agent>`;
    (e) the env contract (`EVAL_MODEL`, `EVAL_TASK_ID`, upstream creds) is identical across all three surfaces.
    Benchmarks failing any sub-test cannot be merged. There is no per-benchmark drift check — one chart cannot drift from itself.

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
| 2026-06-03 | Per-benchmark `values.yaml` removed; the benchmark is now selected with `--set benchmark=<x>` and its bespoke topology (if any) lives in `benchmarks/_chart/presets/<x>.yaml`, bundled inside the chart and overlaid via `.Files.Get`. The chart is now self-contained — `helm template … --set benchmark=<x>` needs no external file, so it can be packaged/published to an OCI registry. Rule 24 drops `values.yaml` as a required artifact (k8s works for every benchmark with no per-benchmark file); 24b/24c/24e/25 retarget at the preset; rule 29(a)/(d) drop the values.yaml existence + pin checks and render via `--set benchmark`. The 4 bespoke benchmarks (osworld, tau-bench, visualwebarena, webarena) became presets; the 98 trivial one-line files were deleted. Renders byte-identical to the prior `-f values.yaml` form. |
| 2026-06-08 | Rules 4, 12, 14, 22: replaced stale `dock-entrypoint.sh` / "the entrypoint" references with the framework launcher (`/usr/local/bin/run`) and the process-compose pipeline. Aligns rule text with the eval-entrypoint.sh → run migration. |
| 2026-06-10 | Rule 7 (agent isolation / eval integrity): the agent process MUST NOT receive the task **identity** (`EVAL_TASK_ID`/`TASK_ID`) — a model can recall a memorized solution from a benchmark instance id. Dropped `TASK_ID` from the agent's `env -i` allow-list in `core/process-compose/process-compose.yaml`; the verifier/result steps still read it from the inherited container env, so grading is unaffected. Removed `EVAL_TASK_ID` from rule 7's agent-allowed set. |
| 2026-06-10 | New rule **24f** (eval-image naming): shared-env → `evals/<b>--<a>`, per-task → `evals/<b>-<task>--<a>`, addressed identically across build/compose/container/job. Rule **24a** carve-out: a per-task `container.Dockerfile` is `ARG EVAL_TASK_ID` + a task-aware `FROM` (the ARG is consumed, so not inert). Rule **24c** clarified: per-task benchmarks run one Job per task and MUST NOT use the Indexed dataset Job (the chart rejects `datasetSize` when `perTask`). Fixes `build eval --task-id` / container / job all emitting the task-less name while compose used the task-aware one. Also sanitizes the job-mode Helm release name to a DNS-1123 label (`naming::release_name`): per-task task ids carry `_` (SWE-bench's `sympy__sympy-24066`), which Helm rejects, so `--mode job` previously could not render for any per-task benchmark. |
| 2026-06-10 | Consolidated per-task detection onto one label-based function. `naming::is_per_task` (name→label) and `benchmark::is_per_task` (Dockerfile→`FROM`/`ARG` heuristic) detected the same property two ways; merged into `benchmark::is_per_task(dockerfile)` keyed on the rule-24f `eval.benchmark.env="per-task"` label, plus a thin `is_per_task_by_name` wrapper, with run / oracle / conformance / build-test all routing through it. A `per_task_label_matches_structure` conformance test keeps the old structural heuristic as a lint and asserts label set == heuristic set across the catalog (catching a per-task Dockerfile that forgot the label). |
| 2026-06-10 | New rule **24g** (per-task environments built from source) + terminal-bench rehabilitated. terminal-bench was non-functional: it didn't build (apt `python3-yaml` vs the base's py3.13 → `ModuleNotFoundError`), never built each task's environment (shared base + copied files, while per-task upstream images don't exist and tasks have heterogeneous bases + setup), and leaked the gold solution + tests into an agent-readable `/task`. Now `build.sh` builds each task's **own** upstream Dockerfile → task env, then the benchmark `Dockerfile` overlays the eval pipeline (`FROM ${TASK_BASE}`); the solution is never baked and tests are root-only. `src/build.rs` runs `benchmarks/<name>/build.sh` when present; `solution.sh` fetches the per-task upstream gold fresh (solution.sh + solution.yaml). Oracle gold=1/no-op=0 verified on hello-world, broken-python, analyze-access-logs, assign-seats. |
| 2026-06-14 | New rule **20a** (the oracle's gold solution derives the answer rather than embedding or copying it) and **24i** (a per-task gold solution reads its task id from a baked value, not the runtime `EVAL_TASK_ID`); rule **21** adds `eval.benchmark.data_revision` to the required-label set (rule 3). |
