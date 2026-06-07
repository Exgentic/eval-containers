---
name: add-agent
description: >-
  Use when adding a new agent image to the fleet — the AI system that runs
  *inside* a benchmark, packaging an install script plus an entrypoint that
  reads a task and prints an answer. Walks the two-script contract, the single
  LLM base-URL, the two version knobs, required labels, and the smoke/replay
  tests. For building the task-plus-verifier environment the agent runs against,
  use add-benchmark instead.
---

# Add an agent

An agent image packages an AI system for evaluation. It provides an
`install.sh` that sets up the runtime and an `entrypoint.sh` that reads a task
and produces an answer. This skill produces a new agent that satisfies
`doctrine/agents/RULES.md`. Read that RULES.md before starting. A copyable
starting point lives at [`assets/TEMPLATE.md`](assets/TEMPLATE.md) — the steps
below tell you how to use it and why each piece exists.

## Steps

1. **Create the agent directory and copy the template.** Make `agents/<name>/`
   and copy [`assets/TEMPLATE.md`](assets/TEMPLATE.md) as your scaffold. The
   Dockerfile must emit the two required scripts at fixed paths:
   `/opt/agent/install.sh` and `/agent/run.sh`. *Why:* every agent
   MUST provide exactly these two scripts — install sets up the runtime,
   entrypoint runs the agent (`doctrine/agents/RULES.md:1`). For the simplest
   possible example see `agents/raw/Dockerfile`; for one needing proxy config
   see `agents/codex/Dockerfile`.

2. **Write `install.sh` to work on any benchmark base.** It runs *inside the
   benchmark image at build time*, which could be `python:3.12-slim`,
   `ubuntu:24.04`, or an upstream image — so it MUST handle missing packages and
   MUST NOT assume a specific OS or language runtime. *Why:*
   `doctrine/agents/RULES.md:11` (install on any base). The agent layer sits on
   top of the benchmark base and MUST NOT modify benchmark-provided files
   (`doctrine/agents/RULES.md:15`).

3. **Pin the upstream version (knob 1 of 2 — build-time default).** Declare
   `ARG <NAME>_VERSION=<semver>` as the default, install that exact version (no
   floating tags — `pip install my-agent==1.2.3`, not `pip install my-agent`),
   and record it in the `eval.agent.version` label. *Why:* the image MUST
   produce a reproducible run with no env vars set
   (`doctrine/agents/RULES.md:12`).

4. **Honor the runtime version override (knob 2 of 2 — `EVAL_AGENT_VERSION`).**
   The entrypoint MUST read `EVAL_AGENT_VERSION` and, when set, install and
   activate that upstream version in place of the default before handing control
   to the agent, writing the resolved version to `/output/agent/version.json`
   first. When unset, the build-time default applies unchanged. *Why:*
   `doctrine/agents/RULES.md:13`. A `/opt/agent-cache` volume MAY be used to
   avoid reinstall cost. Note `EVAL_AGENT_TAG` selects the image tag to pull —
   that is Docker's job, not the entrypoint's.

5. **Read the task from `$TASK` and print the answer to stdout.** The entrypoint
   MUST read the task from the `TASK` env var and MUST NOT assume any other
   context about the benchmark; it MUST print the answer to stdout and MUST NOT
   write results to files or specific paths. *Why:* `doctrine/agents/RULES.md:2`
   (input is `$TASK`), `doctrine/agents/RULES.md:3` (output is stdout),
   `doctrine/agents/RULES.md:4` (benchmark-agnostic — the same image MUST work
   with any compatible benchmark). Logging steps to stderr is optional but
   helpful (captured to `/output/agent/stderr.log`).

6. **Route all LLM calls through the gateway via one base-URL env var.** Pick
   exactly one protocol and read exactly one base-URL var; pass it through to
   your SDK *unmodified* (no manual `/v1` appending). The agent MUST NOT call
   LLM providers directly. *Why:* `doctrine/agents/RULES.md:5`. The framework
   sets:

   | Protocol | Env var the agent reads | Set to |
   |---|---|---|
   | Anthropic | `ANTHROPIC_BASE_URL` | `http://gateway:4000/anthropic` |
   | OpenAI | `OPENAI_BASE_URL` | `http://gateway:4000/openai/v1` |
   | Google | `GOOGLE_GEMINI_BASE_URL` | `http://gateway:4000/genai` |

   If your SDK ignores the env var, write a config file in the entrypoint that
   points to it.

7. **Never embed credentials; use placeholder keys.** The image MUST NOT contain
   real API keys. The framework sets placeholders (`ANTHROPIC_API_KEY=sk-proxy`,
   `OPENAI_API_KEY=sk-proxy`, `GEMINI_API_KEY=sk-proxy`) so SDKs boot; the
   gateway holds the real upstream credentials. If your SDK needs a key var not
   in that list, the entrypoint SHOULD set it to `sk-proxy` directly. *Why:*
   `doctrine/agents/RULES.md:6`.

8. **Run unprivileged with no self-sandboxing and no self-timeout.** The agent
   runs as a non-root user and MUST NOT assume root; it MAY write only to
   `/app/` and `/tmp/` and MUST NOT access `/tasks/`, `/tests/`, `/logs/`, or
   `/output/task/`. Do NOT add bubblewrap, seccomp, or any internal sandbox —
   Docker is the sandbox — and do NOT implement your own timeout; the entrypoint
   enforces `EVAL_TIMEOUT`. *Why:* `doctrine/agents/RULES.md:7` (unprivileged),
   `doctrine/agents/RULES.md:8` (limited filesystem),
   `doctrine/agents/RULES.md:9` (external timeout),
   `doctrine/agents/RULES.md:10` (no self-sandboxing).

9. **Set the required labels.** Every agent image MUST carry `eval.type`,
   `eval.agent.name`, `eval.agent.description`, and `eval.agent.version`. *Why:*
   `doctrine/agents/RULES.md:14`.

10. **Add the tests, and clear the smoke gate.** Provide a build test
    (Dockerfile builds + correct labels, `doctrine/agents/RULES.md:16`) and join
    at least one end-to-end replay test with a recorded fixture so the agent runs
    against real model responses without API keys
    (`doctrine/agents/RULES.md:17`). The agent MUST pass `tests/agents/test.rs`:
    boot from the `evals/agents-smoke--<name>` carrier and make at least one LLM
    call to the protocol-namespaced gateway endpoint within `FIRST_CALL_TIMEOUT`
    seconds (the smoke test uses a `models/replay` mock LLM, so no upstream
    credentials are needed). *Why:* `doctrine/agents/RULES.md:18`. If the agent
    cannot satisfy this contract, list it in `tests/agents/broken.md` with the
    root cause and smallest viable fix; removing it from `broken.md` is the
    success condition.

## Rules served

- `doctrine/agents/RULES.md:1-18` — the agent
  contract this skill produces (two scripts, `$TASK` in / stdout out,
  benchmark-agnostic, single-protocol gateway access, no embedded credentials,
  unprivileged + no self-sandbox, the two version knobs, labels, and the
  build/replay/smoke tests).

## References

- [`assets/TEMPLATE.md`](assets/TEMPLATE.md) — copyable Dockerfile scaffold with
  the two scripts, the blanks-to-fill table, and gotchas.
- `doctrine/agents/RULES.md` — the outcomes every agent MUST satisfy.
- `agents/raw/Dockerfile` — simplest possible agent.
- `agents/codex/Dockerfile` — agent that needs proxy config.
