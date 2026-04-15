# openclaw

OpenClaw: general-purpose open-source AI agent with pluggable providers.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [openclaw/openclaw](https://github.com/openclaw/openclaw) |
| Version | `2026.4.11` |
| Install mechanism | npm (`openclaw`) |
| Language runtime | Node.js 22 |

## What it does

OpenClaw is a general-purpose agent CLI with a JSON configuration model and a pluggable provider system. It supports OpenAI-compatible providers via its `openai-completions` API type, which is how Dock wires it to the LiteLLM proxy. Strength: flexible tool/approval policy you can fully declare in config.

## How Dock runs it

The entrypoint writes `~/.openclaw/openclaw.json` declaring a `dock` provider whose `baseUrl` is `$OPENAI_BASE_URL`, and `~/.openclaw/exec-approvals.json` that sets every agent's exec policy to `security: full, ask: off` so tool calls run without prompts. It then runs `openclaw agent --local --message "$TASK" --output text`. Output is plain text on stdout.

## Version

Pinned to `2026.4.11` at image build time (CalVer). Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
