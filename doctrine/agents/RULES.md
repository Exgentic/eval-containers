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

13. **Version is a build arg.** The upstream CLI version MUST be a single `ARG AGENT_VERSION=<pin>` that drives **both** the install and the `eval.agent.version` label, so the label can never disagree with what was installed. Override at build (`build agent --agent-version <x>`); unset uses the pin. The version is immutable per image — there is no runtime override (reproducibility: the running version is whatever the image was built with). `EVAL_AGENT_TAG` selects which image tag to pull — that's Docker's job. Because the combination image re-runs the agent's `install.sh`, the agent MUST publish its version to `/opt/agent/VERSION` (from the ARG) so the combination installs the same version.

14. **Labels.** Every agent image MUST include labels: `eval.type`, `eval.agent.name`, `eval.agent.description`, `eval.agent.version`.

### Combination

15. **Build-time integration.** Agents are combined with benchmarks at build time via the combination Dockerfile. The agent layer sits on top of the benchmark base. The agent MUST NOT modify benchmark-provided files.

### Testing

16. **Build test.** Every agent image MUST have a build test that verifies the Dockerfile builds and produces correct `eval-containers.*` labels.

17. **Replay test.** Every agent MUST participate in at least one end-to-end replay test with a recorded fixture. This verifies the agent runs correctly against real model responses without API keys.

18. **Smoke test.** Every agent MUST pass `tests/agents/test.rs` — boot from the `evals/agents-smoke--<name>` carrier and make at least one LLM call to the protocol-namespaced gateway endpoint within `FIRST_CALL_TIMEOUT` seconds. The smoke test runs with a `models/replay` mock LLM, so no upstream credentials are needed. An agent that cannot satisfy this contract (because its design hardcodes a vendor backend, requires interactive setup, or runs a self-hosted multi-process stack) MUST be listed in `tests/agents/broken.md` with the root cause + smallest viable fix. Removing an agent from `broken.md` is the success condition.

### Tool Access

19. **MCP is benchmark-declared, agent-rendered.** A benchmark that exposes tools to the agent declares its MCP servers in the `EVAL_MCP_SERVERS` environment variable: a JSON object mapping a logical server name to its address, e.g. `{"tools":"http://tools:8000/mcp","db":"http://db:8000/mcp"}`. The servers themselves are benchmark-owned sidecars on the `internal` network ([compose](../compose/RULES.md) rule 12) — the framework provides no shared MCP service, because which tools a task needs is a property of the task.

    An agent that supports MCP MUST read `EVAL_MCP_SERVERS` in its entrypoint and render **every** entry into its own CLI's configuration dialect, over streamable HTTP. Rendering is the agent's own business and MUST stay inside `agents/<name>/` — there is no shared translator, because a translator that knew all the dialects would make every new agent an edit to `core/`. An empty or unset value MUST leave the agent's invocation unchanged, so a benchmark that declares no servers behaves exactly as before.

    An agent that implements this MUST declare it by writing `true` to `/opt/agent/MCP` (the rule 13 mechanism: `/opt/agent/` is the only channel from the agent image into the combination image). An agent that has not implemented it MUST NOT write the file. The framework entrypoint MUST refuse to start a run that pairs a benchmark declaring MCP servers with an agent that has not declared support, and MUST fail it as a framework error rather than a scored result — an agent silently deprived of the tools a task requires scores 0, which is indistinguishable from a genuine failure and would otherwise be averaged into results as if it meant something.

20. **MCP smoke test.** Every agent that declares MCP support MUST pass `tests/agents/mcp.rs` — boot from the `evals/agents-smoke--<name>` carrier with `EVAL_MCP_SERVERS` pointing at `core/mcp-mock` and fetch the server's tool list within `TOOLS_LIST_TIMEOUT` seconds. The assertion is the `tools/list` request observed **at the server**, because `tools/list` is part of the MCP initialization handshake and therefore fires before and independently of any model decision — the test needs no inference and no tokens. Server-side observation is required rather than parsing agent logs: every MCP client in the fleet treats a failed server as non-fatal and continues silently, so a misconfigured server is invisible from the agent's side. An agent whose upstream CLI cannot satisfy this contract MUST be listed in `tests/agents/mcp-broken.md` with the root cause and smallest viable fix.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 12 into rule 12 (reproducible by default via pinned `ARG <NAME>_VERSION`) and new rule 13 (runtime override via `EVAL_AGENT_VERSION`, writes resolved version to `/output/agent/version.json`). Added `eval.agent.version` to required labels (rule 14). Renumbered rules 14–17. |
| 2026-05-21 | Added rule 18 (smoke test) — agents must pass `tests/agents/test.rs` or be documented in `tests/agents/broken.md`. |
| 2026-08-18 | Added section "Tool Access" with rule 19 (MCP is benchmark-declared via `EVAL_MCP_SERVERS`, rendered per-agent, capability published to `/opt/agent/MCP`, mismatched pairing fails as a framework error) and rule 20 (MCP smoke test — `tests/agents/mcp.rs` asserts the `tools/list` handshake server-side, no inference required). |
