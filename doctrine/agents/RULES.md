# Agents

**Status:** Active
**Date:** April 2026

## Abstract

An agent image packages an AI system for evaluation. It provides an installation script and an entrypoint that reads a task and produces an answer. This document defines the requirements for agent images in Eval Containers.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Contract

1. **Two scripts.** Every agent MUST provide `/opt/agent/install.sh` and `/run.sh`. Install sets up the runtime. Entrypoint runs the agent.

2. **Input is `$TASK`.** The entrypoint MUST read the task from the `TASK` environment variable. The agent MUST NOT assume any other context about the benchmark.

3. **Output is stdout.** The agent MUST print its answer to stdout. The entrypoint captures it. The agent MUST NOT write results to files or specific paths.

4. **Benchmark-agnostic.** The agent MUST NOT know which benchmark it is running in. The same agent image MUST work with any compatible benchmark.

### LLM Access

5. **One protocol, one URL.** Each agent uses exactly one LLM protocol (Anthropic / OpenAI / Google) and reads exactly one base-URL env var. The framework sets each var to the gateway's protocol-namespaced endpoint, matching the SDK's conventional base-URL shape:

    | Protocol | Env var the agent reads | Framework sets it to |
    |---|---|---|
    | Anthropic | `ANTHROPIC_BASE_URL` | `http://gateway:4000/anthropic` (bare host — Anthropic SDK appends `/v1/messages`) |
    | OpenAI | `OPENAI_BASE_URL` | `http://gateway:4000/openai/v1` (with `/v1` — OpenAI SDK appends `/chat/completions`) |
    | Google | `GOOGLE_GEMINI_BASE_URL` | `http://gateway:4000/genai` (bare host — Gemini SDK appends `/v1beta/models/{m}:generateContent`) |

    Each value matches what the upstream provider's official SDK uses as `base_url` by default. The agent MUST pass the env var through to its SDK unmodified — no manual `/v1` appending or path manipulation. The agent MUST NOT call LLM providers directly.

6. **No embedded credentials.** The agent image MUST NOT contain real API keys. The framework sets placeholder values (`ANTHROPIC_API_KEY=sk-proxy`, `OPENAI_API_KEY=sk-proxy`, `GEMINI_API_KEY=sk-proxy`) so SDKs boot; the gateway holds the real upstream credentials. If the agent's SDK requires a key variable not in this list, the entrypoint SHOULD set it to `sk-proxy` directly.

### Constraints

7. **Unprivileged.** The agent runs as a non-root user. It MUST NOT assume root access.

8. **Limited filesystem.** The agent MAY write to `/app/` and `/tmp/`. It MUST NOT access `/tasks/`, `/tests/`, `/logs/`, or `/output/task/`.

9. **External timeout.** The entrypoint enforces `EVAL_TIMEOUT`. The agent MUST NOT implement its own timeout.

10. **No self-sandboxing.** The agent MUST NOT manage its own permissions or sandbox. Docker is the sandbox. The agent SHOULD run with full permissions inside the container — no bubblewrap, no seccomp, no internal sandboxing. Isolation is the container's job, not the agent's.

### Portability

11. **Install on any base.** `install.sh` MUST work on any benchmark base image. It MUST handle missing packages and MUST NOT assume a specific OS or language runtime.

12. **Reproducible by default.** The upstream CLI version MUST be pinned at build time as a default in the Dockerfile (`ARG <NAME>_VERSION=<semver>`) and recorded in `eval.agent.version`. The image MUST produce a reproducible run with no environment variables set.

13. **Runtime version override.** The entrypoint MUST read `EVAL_AGENT_VERSION` and, when set, install and activate that upstream version in place of the default before handing control to the agent. The entrypoint MUST write the resolved version to `/output/agent/version.json` before the agent starts. When `EVAL_AGENT_VERSION` is unset, the build-time default applies unchanged. Cache volumes (`/opt/agent-cache`) MAY be used to avoid reinstall cost on subsequent runs. `EVAL_AGENT_TAG` selects which container version (image tag) to pull — that's Docker's job, not the entrypoint's.

14. **Labels.** Every agent image MUST include labels: `eval.type`, `eval.agent.name`, `eval.agent.description`, `eval.agent.version`.

### Combination

15. **Build-time integration.** Agents are combined with benchmarks at build time via the combination Dockerfile. The agent layer sits on top of the benchmark base. The agent MUST NOT modify benchmark-provided files.

### Testing

16. **Build test.** Every agent image MUST have a build test that verifies the Dockerfile builds and produces correct `eval-containers.*` labels.

17. **Replay test.** Every agent MUST participate in at least one end-to-end replay test with a recorded fixture. This verifies the agent runs correctly against real model responses without API keys.

18. **Smoke test.** Every agent MUST pass `tests/agents/test.rs` — boot from the `evals/agents-smoke--<name>` carrier and make at least one LLM call to the protocol-namespaced gateway endpoint within `FIRST_CALL_TIMEOUT` seconds. The smoke test runs with a `models/replay` mock LLM, so no upstream credentials are needed. An agent that cannot satisfy this contract (because its design hardcodes a vendor backend, requires interactive setup, or runs a self-hosted multi-process stack) MUST be listed in `tests/agents/broken.md` with the root cause + smallest viable fix. Removing an agent from `broken.md` is the success condition.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 12 into rule 12 (reproducible by default via pinned `ARG <NAME>_VERSION`) and new rule 13 (runtime override via `EVAL_AGENT_VERSION`, writes resolved version to `/output/agent/version.json`). Added `eval.agent.version` to required labels (rule 14). Renumbered rules 14–17. |
| 2026-05-21 | Added rule 18 (smoke test) — agents must pass `tests/agents/test.rs` or be documented in `tests/agents/broken.md`. |
