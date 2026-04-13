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

3. **Reproducible.** The exact dataset version MUST be pinned. The image MUST produce identical task content on every build from the same source.

### Isolation

4. **Least privilege.** The agent MUST have access only to what it needs to perform the task — nothing more. Evaluation code, grading logic, expected answers, rubrics, and test infrastructure MUST be inaccessible to the agent. The agent MUST NOT be able to read or modify anything used by the test phase.

5. **Simplest isolation.** Use the simplest mechanism that achieves the required isolation. File permissions over separate containers. User separation over network policies. Complexity is the enemy of security.

6. **Minimal agent environment.** Only `TASK`, `TASK_ID`, `OPENAI_BASE_URL`, and `ANTHROPIC_BASE_URL` SHOULD reach the agent process. Benchmark internals MUST NOT leak.

7. **No credentials.** The eval container MUST NOT contain API keys. LLM credentials MUST exist only in the model service.

8. **No internet by default.** The eval container MUST NOT have outbound internet access unless the benchmark explicitly requires it.

9. **Resource limits.** Every benchmark MUST specify CPU and memory limits in its compose file.

10. **Docker-native security.** Isolation MUST use standard Docker features only (networks, capabilities, read-only filesystem, tmpfs, resource limits). Dock MUST NOT invent security abstractions. If Docker can't enforce it, Dock doesn't promise it.

### Execution

11. **Three-phase flow.** Execution MUST follow: agent runs → test runs → result is written. The shared `dock-entrypoint.sh` handles this. Benchmarks MUST NOT bypass it.

12. **Agent as non-root.** The agent MUST run as an unprivileged user. The test phase MAY run as root.

13. **Timeout.** Agent execution MUST be bounded by `DOCK_TIMEOUT`. The entrypoint enforces this.

### Task Format

14. **Stable task IDs.** For shared-env benchmarks, tasks MUST be addressable by sequential integers (`0`, `1`, `2`, ...) with the original upstream identifier stored in `id.txt`. For per-task benchmarks, the upstream identifier is the task ID. Per-task benchmarks SHOULD publish a `tasks.txt` file (one ID per line) so integers can be mapped to original IDs.

15. **Flat files.** Task data SHOULD be stored as plain files (`problem.txt`, `answer.txt`). No databases, no archive formats.

16. **Agent-visible files.** If the agent needs attached files (documents, images), they MUST be placed in a location the agent can read (e.g., `/app/`). The benchmark MUST NOT give the agent read access to the full task store.

### Scoring

17. **Reward contract.** The test script MUST write a reward to `/logs/verifier/reward.txt`. The value MUST be a float in `[0.0, 1.0]`, or `-1` for externally graded benchmarks.

18. **Simplest correct scorer.** Benchmarks SHOULD use the simplest scoring method that produces correct results. Exact match when possible, code execution for programming, LLM-as-judge only when nothing simpler works.

19. **External grading.** If scoring requires an outside service, the benchmark MUST still collect the agent's output. It MUST write `-1` as the reward. It MUST NOT approximate the external grader.

### Image

20. **Labels.** Every benchmark image MUST include labels: `dock.type`, `dock.benchmark.name`, `dock.benchmark.description`, `dock.benchmark.tasks`, `dock.benchmark.env`, `dock.benchmark.internet`.

21. **Shared components.** Benchmarks SHOULD use shared core images (`dock-entrypoint.sh`, `test-exact-match`) when applicable. Benchmarks MUST NOT reimplement shared logic.

22. **No agent tooling.** Benchmark images MUST NOT include agent-specific tools (browsers, automation libraries, SDKs). The agent's `install.sh` installs what it needs. The benchmark provides the environment, not the tools.

### Compose

23. **One compose per benchmark.** Each benchmark MUST have exactly one compose file, parameterized by `TASK_ID`, `DOCK_AGENT`, and `DOCK_MODEL`.

24. **Extend shared services.** Compose files MUST extend model and eval base definitions from `compose/services.yaml`.

### Testing

25. **Build test.** Every benchmark image MUST have a build test that verifies the Dockerfile builds and produces correct `dock.*` labels.

26. **Compose test.** Every benchmark MUST have a compose validation test that verifies `docker compose config` succeeds.

27. **Replay test.** Every benchmark MUST have at least one end-to-end test using the replay model with a recorded fixture. This test MUST verify that `result.json` is produced with the correct schema.

## References

- [Process](../RULES.md)
- [Agents](../agents/RULES.md)
- [Repository](../compose/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
