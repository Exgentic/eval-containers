# Agents

**Status:** Active
**Date:** April 2026

## Abstract

An agent image packages an AI system for evaluation. It provides an installation script and an entrypoint that reads a task and produces an answer. This document defines the requirements for agent images in Eval Containers.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Contract

1. **Two scripts.** Every agent MUST provide `/opt/agent/install.sh` to set up the runtime and `/opt/agent/entrypoint.sh` to run the agent.

2. **Input is `$TASK`.** The entrypoint MUST read the task from the `TASK` environment variable and MUST NOT assume any other context about the benchmark.

3. **Output is stdout.** The agent MUST print its answer to stdout and MUST NOT write results to files or specific paths.

4. **Benchmark-agnostic.** The agent MUST NOT know which benchmark it runs in, and the same image MUST work with any compatible benchmark.

### LLM Access

5. **One protocol, one URL.** Each agent uses exactly one LLM protocol (Anthropic / OpenAI / Google) and reads exactly one base-URL env var. The framework sets each var to the gateway's protocol-namespaced endpoint, matching the SDK's conventional base-URL shape:

    | Protocol | Env var the agent reads | Framework sets it to |
    |---|---|---|
    | Anthropic | `ANTHROPIC_BASE_URL` | `http://gateway:4000/anthropic` (bare host — Anthropic SDK appends `/v1/messages`) |
    | OpenAI | `OPENAI_BASE_URL` | `http://gateway:4000/openai/v1` (with `/v1` — OpenAI SDK appends `/chat/completions`) |
    | Google | `GOOGLE_GEMINI_BASE_URL` | `http://gateway:4000/genai` (bare host — Gemini SDK appends `/v1beta/models/{m}:generateContent`) |

    The agent MUST pass the env var through to its SDK unmodified and MUST NOT call LLM providers directly.

6. **No embedded credentials.** The agent image MUST NOT contain real API keys; the framework sets `sk-proxy` placeholders so SDKs boot, and the entrypoint SHOULD set any other required key variable to `sk-proxy`.

### Constraints

7. **Unprivileged.** The agent MUST NOT assume root access.

8. **Limited filesystem.** The agent MAY write to `/app/` and `/tmp/`, and MUST NOT access `/tasks/`, `/tests/`, `/logs/`, or `/output/task/`.

9. **External timeout.** The agent MUST NOT implement its own timeout; the entrypoint enforces `EVAL_TIMEOUT`.

10. **No self-sandboxing.** The agent MUST NOT manage its own permissions or sandbox and SHOULD run with full permissions inside the container.

### Portability

11. **Install on any base.** `install.sh` MUST work on any benchmark base image, MUST handle missing packages, and MUST NOT assume a specific OS or language runtime.

12. **Reproducible by default.** The upstream CLI version MUST be pinned at build time as a default in the Dockerfile and recorded in `eval.agent.version`, and the image MUST produce a reproducible run with no environment variables set.

13. **Runtime version override.** The entrypoint MUST read `EVAL_AGENT_VERSION` and, when set, install and activate that upstream version in place of the default and write the resolved version to `/output/agent/version.json` before the agent starts.

14. **Labels.** Every agent image MUST include labels `eval.type`, `eval.agent.name`, `eval.agent.description`, and `eval.agent.version`.

### Combination

15. **Build-time integration.** Agents are combined with benchmarks at build time via the combination Dockerfile, with the agent layer on top of the benchmark base, and the agent MUST NOT modify benchmark-provided files.

### Testing

16. **Build test.** Every agent image MUST have a build test verifying the Dockerfile builds and produces correct `eval-containers.*` labels.

17. **Replay test.** Every agent MUST participate in at least one end-to-end replay test with a recorded fixture.

18. **Smoke test.** Every agent MUST pass `tests/agents/test.rs` by booting from the `evals/agents-smoke--<name>` carrier and making at least one LLM call to the protocol-namespaced gateway endpoint within `FIRST_CALL_TIMEOUT` seconds, or MUST be listed in `tests/agents/broken.md` with the root cause and smallest viable fix.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 12 into rule 12 (reproducible by default via pinned `ARG <NAME>_VERSION`) and new rule 13 (runtime override via `EVAL_AGENT_VERSION`, writes resolved version to `/output/agent/version.json`). Added `eval.agent.version` to required labels (rule 14). Renumbered rules 14–17. |
| 2026-05-21 | Added rule 18 (smoke test) — agents must pass `tests/agents/test.rs` or be documented in `tests/agents/broken.md`. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
