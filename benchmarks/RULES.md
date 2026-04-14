# Benchmarks

**Status:** Active
**Date:** April 2026

## Abstract

A benchmark image contains everything needed to evaluate an agent on a task. This document defines the requirements for building benchmark images in Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Self-Contained

1. **Standalone.** The image MUST contain all task data, test logic, and entrypoints. `docker compose up` MUST work without Dock installed and without internet. For shared-env benchmarks, `TASK_ID` is the only required runtime input. For per-task benchmarks, `TASK_ID` is a build-time argument — each image contains exactly one task.

2. **Single input.** The image MUST resolve to the task content, expected answer, and any attached files from `TASK_ID` alone — whether provided at build time (per-task) or runtime (shared-env).

3. **Reproducible by default.** The exact dataset version MUST be pinned at build time as a default in the Dockerfile (`ARG DATA_REVISION=<sha>` or equivalent) and recorded in `dock.benchmark.data_revision`. The image MUST produce identical task content on every build when no env vars are set.

4. **Runtime version override.** The entrypoint MUST read `DOCK_BENCHMARK_VERSION` and, when set, fetch and materialize that dataset revision into `/tasks/` in place of the default. It MUST write the resolved revision to `/output/task/version.json` before the agent runs. When `DOCK_BENCHMARK_VERSION` is unset, the build-time default applies unchanged.

### Isolation

5. **Least privilege.** The agent MUST have access only to what it needs to perform the task — nothing more. Evaluation code, grading logic, expected answers, rubrics, and test infrastructure MUST be inaccessible to the agent. The agent MUST NOT be able to read or modify anything used by the test phase.

6. **Simplest isolation.** Use the simplest mechanism that achieves the required isolation. File permissions over separate containers. User separation over network policies. Complexity is the enemy of security.

7. **Minimal agent environment.** Only `TASK`, `DOCK_TASK_ID`, `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent process. Benchmark internals MUST NOT leak.

8. **No credentials.** The eval container MUST NOT contain API keys. LLM credentials MUST exist only in the model service.

9. **No internet by default.** The eval container MUST NOT have outbound internet access unless the benchmark explicitly requires it.

10. **Resource limits.** Every benchmark MUST specify CPU and memory limits in its compose file.

11. **Docker-native security.** Isolation MUST use standard Docker features only (networks, capabilities, read-only filesystem, tmpfs, resource limits). Dock MUST NOT invent security abstractions. If Docker can't enforce it, Dock doesn't promise it.

### Execution

12. **Three-phase flow.** Execution MUST follow: agent runs → test runs → result is written. The shared `dock-entrypoint.sh` handles this. Benchmarks MUST NOT bypass it.

13. **Agent as non-root.** The agent MUST run as an unprivileged user. The test phase MAY run as root.

14. **Timeout.** Agent execution MUST be bounded by `DOCK_TIMEOUT`. The entrypoint enforces this.

### Task Format

15. **Stable task IDs.** For shared-env benchmarks, tasks MUST be addressable by sequential integers (`0`, `1`, `2`, ...) with the original upstream identifier stored in `id.txt`. For per-task benchmarks, the upstream identifier is the task ID. Per-task benchmarks SHOULD publish a `tasks.txt` file (one ID per line) so integers can be mapped to original IDs.

16. **Flat files.** Task data SHOULD be stored as plain files (`problem.txt`, `answer.txt`). No databases, no archive formats.

17. **Agent-visible files.** If the agent needs attached files (documents, images), they MUST be placed in a location the agent can read (e.g., `/app/`). The benchmark MUST NOT give the agent read access to the full task store.

### Scoring

18. **Reward contract.** The test script MUST write a reward to `/logs/verifier/reward.txt`. The value MUST be a float in `[0.0, 1.0]`, or `-1` for externally graded benchmarks.

19. **Simplest correct scorer.** Benchmarks SHOULD use the simplest scoring method that produces correct results. Exact match when possible, code execution for programming, LLM-as-judge only when nothing simpler works.

20. **External grading.** If scoring requires an outside service, the benchmark MUST still collect the agent's output. It MUST write `-1` as the reward. It MUST NOT approximate the external grader.

### Image

21. **Labels.** Every benchmark image MUST include labels: `dock.type`, `dock.benchmark.name`, `dock.benchmark.description`, `dock.benchmark.tasks`, `dock.benchmark.env`, `dock.benchmark.internet`.

22. **Shared components.** Benchmarks SHOULD use shared core images (`dock-entrypoint.sh`, `test-exact-match`) when applicable. Benchmarks MUST NOT reimplement shared logic.

23. **No agent tooling.** Benchmark images MUST NOT include agent-specific tools (browsers, automation libraries, SDKs). The agent's `install.sh` installs what it needs. The benchmark provides the environment, not the tools.

### Compose

24. **One compose per benchmark.** Each benchmark MUST have exactly one compose file, parameterized by `DOCK_TASK_ID`, `DOCK_AGENT`, and `DOCK_MODEL`.

25. **Extend shared services.** Compose files MUST extend model and eval base definitions from `compose/services.yaml`.

### Testing

26. **Build test.** Every benchmark image MUST have a build test that verifies the Dockerfile builds and produces correct `dock.*` labels.

27. **Compose test.** Every benchmark MUST have a compose validation test that verifies `docker compose config` succeeds.

28. **Replay test.** Every benchmark MUST have at least one end-to-end test using the replay model with a recorded fixture. This test MUST verify that `result.json` is produced with the correct schema.

## References

- [Process](../RULES.md)
- [Agents](../agents/RULES.md)
- [Repository](../compose/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 3 into rule 3 (reproducible by default via pinned `ARG DATA_REVISION`) and new rule 4 (runtime override via `DOCK_BENCHMARK_VERSION`, writes resolved revision to `/output/task/version.json`). Renumbered rules 5–28. Standardized `TASK_ID` → `DOCK_TASK_ID` in the minimal-agent-environment rule. |
