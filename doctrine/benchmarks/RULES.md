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

4. **Runtime version override.** The entrypoint MUST read `EVAL_BENCHMARK_VERSION` and, when set, fetch and materialize that dataset revision into `/tasks/` in place of the default. It MUST write the resolved revision to `/output/task/version.json` before the agent runs. When `EVAL_BENCHMARK_VERSION` is unset, the build-time default applies unchanged. `EVAL_BENCHMARK_TAG` selects which container version (image tag) to pull — that's Docker's job, not the entrypoint's.

### Isolation

5. **Least privilege.** The agent MUST have access only to what it needs to perform the task — nothing more. Evaluation code, grading logic, expected answers, rubrics, and test infrastructure MUST be inaccessible to the agent. The agent MUST NOT be able to read or modify anything used by the test phase.

6. **Simplest isolation.** Use the simplest mechanism that achieves the required isolation. File permissions over separate containers. User separation over network policies. Complexity is the enemy of security.

7. **Minimal agent environment.** Only `TASK`, `EVAL_TASK_ID`, `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent process. Benchmark internals MUST NOT leak.

8. **No agent access to credentials.** The agent process MUST NOT have access to LLM credentials. In compose-mode benchmarks, credentials live in the separate model service. In single-image benchmarks, credentials live in a 0600 file owned by a proxy UID inaccessible to the agent UID. Both implementations MUST achieve the same property: the agent process cannot read the API key.

9. **No agent internet by default.** The agent MUST NOT have outbound internet access unless the benchmark explicitly requires it. In compose-mode benchmarks, this is enforced by `internal: true` on the agent's network. In single-image benchmarks, by `iptables -m owner --uid-owner` rules on the agent UID. Both implementations MUST achieve the same property: the agent has no path to the open internet by default.

10. **Resource limits.** Every benchmark MUST specify CPU and memory limits in its compose file.

11. **Docker-native security.** Isolation MUST use standard Docker features only (networks, capabilities, read-only filesystem, tmpfs, resource limits). Dock MUST NOT invent security abstractions. If Docker can't enforce it, Dock doesn't promise it.

### Execution

12. **Three-phase flow.** Execution MUST follow: agent runs → test runs → result is written. The shared `dock-entrypoint.sh` handles this. Benchmarks MUST NOT bypass it.

13. **Agent as non-root.** The agent MUST run as an unprivileged user. The test phase MAY run as root.

14. **Timeout.** Agent execution MUST be bounded by `EVAL_TIMEOUT`. The entrypoint enforces this.

### Task Format

15. **Stable task IDs.** For shared-env benchmarks, tasks MUST be addressable by sequential integers (`0`, `1`, `2`, ...) with the original upstream identifier stored in `id.txt`. For per-task benchmarks, the upstream identifier is the task ID. Per-task benchmarks SHOULD publish a `tasks.txt` file (one ID per line) so integers can be mapped to original IDs.

16. **Flat files.** Task data SHOULD be stored as plain files (`problem.txt`, `answer.txt`). No databases, no archive formats.

17. **Agent-visible files.** If the agent needs attached files (documents, images), they MUST be placed in a location the agent can read (e.g., `/app/`). The benchmark MUST NOT give the agent read access to the full task store.

### Scoring

18. **Reward contract.** The test script MUST write a reward to `/logs/verifier/reward.txt`. The value MUST be a float in `[0.0, 1.0]`, or `-1` for externally graded benchmarks.

19. **Simplest correct scorer.** Benchmarks SHOULD use the simplest scoring method that produces correct results. Exact match when possible, code execution for programming, LLM-as-judge only when nothing simpler works.

20. **External grading.** If scoring requires an outside service, the benchmark MUST still collect the agent's output. It MUST write `-1` as the reward. It MUST NOT approximate the external grader.

### Image

21. **Labels.** Every benchmark image MUST include labels: `eval.type`, `eval.benchmark.name`, `eval.benchmark.description`, `eval.benchmark.tasks`, `eval.benchmark.env`, `eval.benchmark.internet`. Benchmarks that have graduated to the release gate MUST also carry `eval.benchmark.released="true"` (see principle 21a).

21a. **Release readiness gate.** A benchmark is **released** when it has been proven end-to-end against at least one agent with a recorded replay fixture under `tests/fixtures/<benchmark>-<task>-<agent>.trajectory.jsonl`. Released benchmarks MUST carry `LABEL eval.benchmark.released="true"`. Unreleased benchmarks MAY exist in `benchmarks/` (the source tree is the full catalog of what Dock COULD support), but MUST NOT carry the label until a fixture lands and they pass the replay sweep. `doctrine/verification/audit-fleet/references/checklist.md` question 3 (replay coverage) checks this label, not the directory count — the filesystem can hold 96 benchmarks while only a subset are released.

21b. **Upstream base tracking.** Benchmarks whose `FROM` line points at a third-party registry not under Dock's control (e.g. `ghcr.io/andyzorigin/*`, `ghcr.io/openai/*`) MUST declare `LABEL eval.benchmark.upstream_base="<full image ref>"` recording the exact upstream reference. This makes the external dependency visible to audit tools and to anyone reading the image metadata. Benchmarks that build from a Dock-controlled or fully in-repo base (e.g. `FROM python:3.12-slim`) do NOT need this label. `doctrine/verification/audit-fleet/references/checklist.md` question 6 (stale upstream images) walks every `upstream_base` label and reports yellow if any still points at `:latest` — such bases are legal but flagged as known supply-chain debt until mirrored or pinned by digest.

22. **Shared components.** Benchmarks SHOULD use shared core images (`dock-entrypoint.sh`, `test-exact-match`) when applicable. Benchmarks MUST NOT reimplement shared logic.

23. **No agent tooling.** Benchmark images MUST NOT include agent-specific tools (browsers, automation libraries, SDKs). The agent's `install.sh` installs what it needs. The benchmark provides the environment, not the tools.

### Three Deployment Surfaces

24. **Triple-mode contract — every benchmark ships exactly three deployment artifacts**, one per surface. The artifacts MUST share the same env contract (`EVAL_MODEL`, `EVAL_TASK_ID`, upstream credentials), MUST produce byte-equivalent `task/result.json` outputs for the same inputs (modulo timestamps and provider-side request IDs), and MUST exercise the same five-unit pipeline (otelcol → gateway → agent → verifier → result).

    | File | Mode | Topology | Invocation |
    |------|------|----------|------------|
    | `container.Dockerfile` | **single** | 1 container, 5 processes inside (process-compose orchestrates) | `docker build -f benchmarks/<x>/container.Dockerfile -t <tag> . && docker run <tag>` |
    | `compose.yaml` | **compose** | 3 services on a compose network (otelcol + gateway + runner) | `docker compose -f benchmarks/<x>/compose.yaml up` |
    | `job.yaml` | **k8s** | Self-contained multi-doc manifest: 1 `Job` + 1 Pod + 3 containers (`shareProcessNamespace` for sidecar reaping; isolation via credentials — see 24d), plus any bespoke `Deployment`s and `Service`s the benchmark needs | `kubectl apply -f benchmarks/<x>/job.yaml` |

    Single mode is the simplest surface (one `docker run`, no orchestrator); compose and k8s split the pipeline across containers so the agent process cannot see upstream credentials (rule 8). Benchmarks that ship only one or two surfaces are incomplete.

24a. **Universal eval-image recipe.** `core/combination.Dockerfile` is the single source of truth for the eval-image build. Per-benchmark `container.Dockerfile` files MUST be a single-line registry pin of the form `FROM <registry>/evals/<benchmark>--<agent>:<tag>` — nothing more. The canonical build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`, `MODEL_IMAGE`) MUST be recorded in the benchmark's `README.md` so CI can rebuild the eval image by invoking `core/combination.Dockerfile` with those args. Declaring inert `ARG` lines that the `FROM` does not consume is forbidden — they look load-bearing but aren't, and they drift. Duplicating the combination Dockerfile body across benchmarks is forbidden — there is exactly one eval-image recipe in the repo.

24b. **Compose uses shared base; k8s inlines for clarity.** The two surfaces use different sharing primitives because their tooling differs:
    - `compose/services.yaml` — shared compose services (`otelcol`, `gateway`, `runner`); per-benchmark `compose.yaml` pulls it in via `include:` and overrides the benchmark-specific bits.
    - `benchmarks/_base/job.yaml` — the **canonical k8s reference** (otelcol + gateway + runner Pod template). Per-benchmark `job.yaml` files are **standalone, self-contained** k8s manifests: each inlines the canonical Pod spec and appends any bespoke `Deployment`s/`Service`s. A CI test (rule 29) asserts the canonical container blocks across all `job.yaml` files match `benchmarks/_base/job.yaml`, catching drift.

    Compose's `include:` saves duplication for the 96 simple benchmarks; k8s's standalone shape keeps complex benchmarks (which add bespoke `Deployment`s/`Service`s) readable in one file. Both keep `service_healthy` ↔ `shareProcessNamespace` reaper, service-name DNS ↔ Pod loopback, and identical env contract in lockstep — changes to either base MUST be reflected in the other in the same commit.

24c. **Task parameterization in deployment artifacts.** Rule 1 makes `EVAL_TASK_ID` the only required runtime input for shared-env benchmarks; the deployment artifacts MUST honor this:
    - **Shared-env**: `compose.yaml` MUST read the task id from the shell environment as `TASK_ID: ${TASK_ID:-0}` (default 0, override via `TASK_ID=42 docker compose up`). Hardcoding a literal task id in `compose.yaml` is forbidden. `job.yaml` ships as a hardcoded template for task 0; multi-task sweeps use kustomize overlays or `sed` substitution — Dock does not ship a parameterized k8s manifest, and that limitation MUST be called out in the benchmark's README.
    - **Per-task**: `EVAL_TASK_ID` is a build-time `ARG` in `container.Dockerfile`; each image bakes exactly one task. The compose and k8s artifacts inherit the baked-in task — they do not parameterize.

24d. **Network isolation across surfaces.** Rule 9 (no agent internet by default) MUST be enforced in every shipped surface, by the mechanism native to that surface:
    - `compose.yaml`: agent's network is `internal: true`; the gateway is the only service joined to a separate `upstream` network.
    - `container.Dockerfile` (single mode): `iptables -m owner --uid-owner` rules on the agent UID, applied at container start.
    - `job.yaml`: containers in a Pod share the network stack, so a per-container egress firewall is not possible; rule 9 is achieved indirectly via rule 8 (the runner container has no API credentials, so even if it reached `api.openai.com` directly the call would fail auth). A `NetworkPolicy` MAY be added for defense in depth.

    Benchmarks that explicitly require agent internet MUST declare `eval.benchmark.internet=true` AND remove the relevant isolation primitive in every surface that ships. Asymmetry (e.g., compose blocks but k8s allows) is forbidden.

24e. **Resource limit parity.** Rule 10 (resource limits) MUST be expressed in both `compose.yaml` (`deploy.resources.limits` on the runner) and `job.yaml` (`resources.limits` on the runner container). The values MUST match modulo k8s unit syntax (`"2"` ↔ `2`, `"2Gi"` ↔ `2147483648`). GPU benchmarks declare via `deploy.resources.reservations.devices[].driver: nvidia` in compose and `resources.limits["nvidia.com/gpu"]` in k8s.

25. **Use the surface's natural sharing approach.**
    - **compose** has native sharing (`include:`/`extends:`). Per-benchmark `compose.yaml` MUST pull `compose/services.yaml` in via `include:` and only declare overrides. Inlining a service/healthcheck/network/volume that already exists in `compose/services.yaml` is forbidden.
    - **k8s** YAML has no native include. Per-benchmark `job.yaml` inlines the canonical otelcol+gateway+runner Pod from `benchmarks/_base/job.yaml`, plus any bespoke `Deployment`s/`Service`s. The canonical sections MUST byte-match the reference (modulo benchmark name, image, env values, and resource limits) — drift is caught by the CI test in rule 29.

    Effect: when the canonical Pod shape evolves, compose changes 1 file; k8s changes the reference plus mechanical sed across the 100 `job.yaml` files (CI fails until they re-converge).

### Testing

26. **Build test.** Every benchmark image MUST have a build test that verifies the Dockerfile builds and produces correct `dock.*` labels.

27. **Compose test.** Every benchmark MUST have a compose validation test that verifies `docker compose config` succeeds.

28. **Replay test.** Every benchmark MUST have at least one end-to-end test using the replay model with a recorded fixture. This test MUST verify that `result.json` is produced with the correct schema.

29. **Triple-mode existence + canonical-drift test.** A CI test MUST walk every directory in `benchmarks/` and assert:
    (a) `container.Dockerfile`, `compose.yaml`, and `job.yaml` all exist;
    (b) `container.Dockerfile` is a single-line `FROM` (rule 24a);
    (c) `docker compose -f compose.yaml config` succeeds (rule 27);
    (d) `job.yaml` validates against k8s schema (kubeconform or equivalent);
    (e) each `job.yaml`'s `otelcol`, `gateway`, and `runner` container blocks match the canonical blocks in `benchmarks/_base/job.yaml` modulo the benchmark-specific fields (benchmark name in metadata/labels, runner image, `BENCHMARK`/`MODEL`/`TIMEOUT` env values, and resource limits) — drift caught here;
    (f) the env contract (`EVAL_MODEL`, `EVAL_TASK_ID`, upstream creds) is identical across all three surfaces.
    Benchmarks failing any sub-test cannot be merged.

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
