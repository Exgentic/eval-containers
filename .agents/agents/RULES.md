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

8. **Limited filesystem.** The agent MAY write to its task working directory (wherever the benchmark's entrypoint places it — e.g. `/app/`, `/testbed/`) and `/tmp/`. It MUST NOT access `/tasks/`, `/tests/`, `/logs/`, or `/output/task/`.

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

18. **Smoke test.** Every agent MUST pass `tests/run/agents/test.rs` — boot from the `evals/agents-smoke--<name>` carrier and make at least one LLM call to the protocol-namespaced gateway endpoint within `FIRST_CALL_TIMEOUT` seconds. The smoke test runs with a `models/replay` mock LLM, so no upstream credentials are needed. An agent that cannot satisfy this contract (because its design hardcodes a vendor backend, requires interactive setup, or runs a self-hosted multi-process stack) MUST be listed in `tests/run/agents/broken.md` with the root cause + smallest viable fix. Removing an agent from `broken.md` is the success condition.

### Internet policy

19. **Internet policy is a capability toggle, not benchmark knowledge.** A supporting agent MUST read only the `EVAL_INTERNET` env var — never a benchmark name — to decide whether to disable its own internet-accessing capabilities and tools, so it does not attempt access that is bound to fail.

20. **Unsupported means warn, not silently ignored.** The runner MUST warn (not reject) a run where `EVAL_INTERNET=false` but the selected agent's `/run.sh` does not reference `EVAL_INTERNET`, since network isolation (rule 21) still holds regardless of agent support — the pairing is valid, just without the agent avoiding calls that are bound to fail.

21. **Tool removal is not the isolation boundary.** `EVAL_INTERNET=false` MUST NOT be documented or relied on as network isolation — that is [benchmarks/RULES.md rule 9](../benchmarks/RULES.md), unconditional on every benchmark, and it already blocks a raw-socket tool like `WebFetch` (removing it just avoids a call bound to fail, per rule 19). The one channel rule 9 cannot reach is the LLM provider's own server-side web-search tool (e.g. `WebSearch`): it is provider-mediated, not a socket the agent opens, so no network policy touches it. Denying it client-side is a temporary mitigation for that specific gap, not a fix — blocking the provider's web-search at the gateway is the real fix and remains future work.

22. **The task text MUST tell the agent when no internet is needed.** When `EVAL_INTERNET=false`, the runner MUST append a plain-language note to `TASK` stating that no internet connection is required and the agent should not attempt to access it, so the agent does not spend turns diagnosing or working around a perceived connectivity failure.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Split rule 12 into rule 12 (reproducible by default via pinned `ARG <NAME>_VERSION`) and new rule 13 (runtime override via `EVAL_AGENT_VERSION`, writes resolved version to `/output/agent/version.json`). Added `eval.agent.version` to required labels (rule 14). Renumbered rules 14–17. |
| 2026-05-21 | Added rule 18 (smoke test) — agents must pass `tests/run/agents/test.rs` or be documented in `tests/run/agents/broken.md`. |
| 2026-08-10 | Rule 8: replaced the hardcoded `/app/` with "its task working directory (wherever the benchmark's entrypoint places it)" — only 39 of 102 benchmarks actually use `/app`; swe-bench stages at `/testbed`. The old wording had already misled one agent image into hardcoding `/app` (#308). |
| 2026-09-06 | Added rules 19–22 (Internet policy): `EVAL_INTERNET` is a capability toggle read verbatim (19); the runner warns, not rejects (20 — corrected from an initial fail-loud draft, since network isolation, rule 21, holds regardless of agent support, and most of the fleet's agents don't yet support denying their own web tools, so failing would have broken existing valid pairings); denying the agent's web tools is not the isolation boundary (rule 9 already covers raw-socket tools like `WebFetch` unconditionally) but is a temporary client-side mitigation for the one channel that boundary can't reach — the LLM provider's own server-side web-search (21); the runner appends a plain-language no-internet-needed note to `TASK` (22). The label/ENV agreement requirement moved to [benchmarks/RULES.md rule 21c](../benchmarks/RULES.md) (#423); rules 21–23 renumbered to 20–22. |
