# ra-aid

RA.Aid: autonomous research-first coding agent with an expert sub-agent.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [ai-christianson/RA.Aid](https://github.com/ai-christianson/RA.Aid) |
| Version | `0.30.2` |
| Install mechanism | pip (`ra-aid`, into `/opt/ra-aid-venv`) |
| Language runtime | Python |

## What it does

RA.Aid ("Research Assistant Aid") is an autonomous coding agent that runs a research phase before acting and can consult a separate "expert" reasoning sub-agent. Both the main agent and the expert are routed through the same OpenAI-compatible endpoint, which Dock points at the LiteLLM proxy.

## How Dock runs it

The entrypoint activates the venv, sets `OPENAI_API_KEY` and `OPENAI_API_BASE=$OPENAI_BASE_URL/v1`, mirrors those onto `EXPERT_OPENAI_API_*` so the expert sub-agent uses the same proxy, and runs `ra-aid --cowboy-mode --provider openai-compatible --model $DOCK_MODEL --expert-provider openai-compatible --expert-model $DOCK_MODEL -m "$TASK"`. `--cowboy-mode` skips shell approval prompts.

## Version

Pinned to `0.30.2` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
