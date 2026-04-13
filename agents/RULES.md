# Agents

**Status:** Active
**Date:** April 2026

## Abstract

An agent image packages an AI system for evaluation. It provides an installation script and an entrypoint that reads a task and produces an answer. This document defines the requirements for agent images in Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Contract

1. **Two scripts.** Every agent MUST provide `/opt/agent/install.sh` and `/opt/agent/entrypoint.sh`. Install sets up the runtime. Entrypoint runs the agent.

2. **Input is `$TASK`.** The entrypoint MUST read the task from the `TASK` environment variable. The agent MUST NOT assume any other context about the benchmark.

3. **Output is stdout.** The agent MUST print its answer to stdout. The entrypoint captures it. The agent MUST NOT write results to files or specific paths.

4. **Benchmark-agnostic.** The agent MUST NOT know which benchmark it is running in. The same agent image MUST work with any compatible benchmark.

### LLM Access

5. **Proxy only.** All LLM calls MUST route through the model service proxy at `OPENAI_BASE_URL` or `ANTHROPIC_BASE_URL`. The agent MUST NOT call LLM providers directly.

6. **No embedded credentials.** The agent image MUST NOT contain API keys. If the agent's SDK requires a key variable to be set, the entrypoint SHOULD provide a dummy value.

### Constraints

7. **Unprivileged.** The agent runs as a non-root user. It MUST NOT assume root access.

8. **Limited filesystem.** The agent MAY write to `/app/` and `/tmp/`. It MUST NOT access `/tasks/`, `/tests/`, `/logs/`, or `/output/task/`.

9. **External timeout.** The entrypoint enforces `DOCK_TIMEOUT`. The agent MUST NOT implement its own timeout.

10. **No self-sandboxing.** The agent MUST NOT manage its own permissions or sandbox. Docker is the sandbox. The agent SHOULD run with full permissions inside the container — no bubblewrap, no seccomp, no internal sandboxing. Isolation is the container's job, not the agent's.

### Portability

11. **Install on any base.** `install.sh` MUST work on any benchmark base image. It MUST handle missing packages and MUST NOT assume a specific OS or language runtime.

12. **Pin versions.** All dependencies MUST be pinned to exact versions. Agent images MUST be reproducible.

13. **Labels.** Every agent image MUST include labels: `dock.type`, `dock.agent.name`, `dock.agent.description`.

### Combination

14. **Build-time integration.** Agents are combined with benchmarks at build time via the combination Dockerfile. The agent layer sits on top of the benchmark base. The agent MUST NOT modify benchmark-provided files.

### Testing

15. **Build test.** Every agent image MUST have a build test that verifies the Dockerfile builds and produces correct `dock.*` labels.

16. **Replay test.** Every agent MUST participate in at least one end-to-end replay test with a recorded fixture. This verifies the agent runs correctly against real model responses without API keys.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
