# continue-cli

Continue CLI (`cn`) — a terminal-native, multi-model coding agent from the Continue.dev team.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [continuedev/continue](https://github.com/continuedev/continue) |
| Version | `1.5.45` |
| Install mechanism | npm (`@continuedev/cli`) |
| Language runtime | Node.js 22 |

## What it does

Continue started as an IDE extension; the `cn` CLI exposes the same agent loop in a terminal. It supports multiple model providers, configurable tool permissions, and a plan/execute pattern. Eval Containers points it at the LiteLLM proxy so every provider backend routes through a single logged endpoint.

## How Eval Containers runs it

The entrypoint wires `cn` to the proxy via the OpenAI-compatible API surface, runs one turn with the task as the user prompt, and captures stdout. `cn` prints its final answer on completion; the evaluator reads that.

## Version

Pinned to `1.5.45` at image build time. Override with `EVAL_AGENT_VERSION=<version>` at build or run time per [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image (extends `core/agent-base-node`)
- `README.md` — this file
