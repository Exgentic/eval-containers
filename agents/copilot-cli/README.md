# copilot-cli

GitHub Copilot CLI: GitHub's terminal coding assistant.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [github/copilot-cli](https://github.com/github/copilot-cli) |
| Version | `1.0.24` |
| Install mechanism | npm (`@github/copilot`) |
| Language runtime | Node.js 22 |

## What it does

GitHub Copilot CLI is the terminal-facing Copilot agent that can read a workspace, plan multi-step edits, and run commands. In Dock it runs in "offline / BYOK" mode — `COPILOT_OFFLINE=true` disables GitHub cloud auth and all LLM traffic is redirected to an OpenAI-compatible endpoint advertised by the LiteLLM proxy.

## How Dock runs it

The entrypoint sets `COPILOT_PROVIDER_BASE_URL=$OPENAI_BASE_URL/v1`, `COPILOT_PROVIDER_API_KEY=$OPENAI_API_KEY`, `COPILOT_OFFLINE=true`, and `COPILOT_MODEL=$COPILOT_MODEL`, then pipes `$TASK` into `copilot --yolo` over stdin. `--yolo` auto-approves all tool calls. Output is printed to stdout.

## Version

Pinned to `1.0.24` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
