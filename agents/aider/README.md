# aider

Aider: AI pair programming tool that edits local git repositories.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [Aider-AI/aider](https://github.com/Aider-AI/aider) |
| Version | `0.86.2` |
| Install mechanism | pip (into `/opt/aider-venv`) |
| Language runtime | Python |

## What it does

Aider is a terminal-based pair programmer that applies LLM-generated edits to files in a local workspace. It speaks any LiteLLM-compatible provider; Dock wires it at the proxy as `openai/<model>`. Strength: focused code editing with structured diff application.

## How Dock runs it

The entrypoint exports `OPENAI_API_KEY` and invokes `aider --yes-always --no-git --no-auto-commits --no-stream --openai-api-base $OPENAI_BASE_URL --model openai/$DOCK_MODEL --message "$TASK"`. The task string comes from `$TASK`, all LLM traffic is routed at `http://model:4000`, and Aider prints its reply to stdout. No extra mounts are required beyond the standard Dock workspace.

## Version

Pinned to `0.86.2` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
