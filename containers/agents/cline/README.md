# cline

Cline — an autonomous coding agent with **Plan/Act modes** and MCP (Model Context Protocol) integration.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [cline/cline](https://github.com/cline/cline) |
| Version | `2.15.0` |
| Install mechanism | npm (`@cline/cli`) |
| Language runtime | Node.js 22 |

## What it does

Cline operates a two-phase workflow: **Plan mode** surveys the workspace and drafts a change plan without mutating the repo; **Act mode** executes the plan step-by-step with read/write/exec tools. MCP integration lets Cline pull in third-party tool servers (filesystem, git, web) alongside its built-ins.

## How Eval Containers runs it

The entrypoint sets the LiteLLM proxy endpoint and API key, then invokes the CLI in non-interactive mode with the task as the user message. Cline writes its final answer to stdout; Eval Containers's evaluator reads the last non-empty line as the submission.

When the launch sets `EVAL_MODEL_CONTEXT_WINDOW` ([agents rules 25–26](../../../.agents/agents/RULES.md)), the entrypoint gives cline's OpenAI model info that `contextWindow`, for both modes, and cline truncates by its own rule within it. A model info already in cline's state is kept, and a `contextWindow` already in it is not overridden; where there is none, cline's own default info is written with the window. It sends no reply cap. The derivation is in the `/run.sh` comment in the [Dockerfile](Dockerfile). Unset, nothing changes.

## Version

Pinned to `2.15.0` at image build time. Override with `EVAL_AGENT_VERSION=<version>` at build or run time per [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image (extends `core/agent-base-node`)
- `README.md` — this file
