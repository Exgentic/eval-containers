# opencode

SST opencode: MIT open-source AI coding agent.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [sst/opencode](https://github.com/sst/opencode) |
| Version | `1.4.3` |
| Install mechanism | npm (`opencode-ai`) |
| Language runtime | Node.js 22 |

## What it does

opencode is SST's open-source terminal coding agent. It supports arbitrary providers declared in `opencode.json` through the Vercel AI SDK; Eval Containers uses `@ai-sdk/openai-compatible` to register the LiteLLM proxy as a `eval-containers` provider and disables the built-in `anthropic`/`openai` providers so nothing escapes the proxy.

## How Eval Containers runs it

The entrypoint writes `~/.config/opencode/opencode.json` with a `eval-containers` provider (`baseURL: $OPENAI_BASE_URL/v1`), allow-listed permissions for edit/bash/webfetch, `autoshare: false`, and both auto-update and model fetching disabled. It then runs `opencode run --model "eval-containers/$MODEL" "$TASK"`. Dummy `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` are set to silence auth prompts.

opencode's reply cap is raised from its own 32000 to `$OPENCODE_MAX_OUTPUT_TOKENS` (128000 unless set), because a reply cut at the cap ends the session. When the launch sets `EVAL_MODEL_CONTEXT` (the model's context window, [agents rule 25](../../../.agents/agents/RULES.md)), the cap becomes min(that, window/4), so every request fits the window (32,768 on a 131k model, where 128000 rejects the first call). The window sizes only the cap. `OPENCODE_DISABLE_AUTOCOMPACT=1` is set, but in 1.4.3 it only turns off the proactive token-count trigger: a prompt past window − cap draws the provider's context-overflow error, and opencode answers that by summarizing the conversation and continuing. Unset, nothing changes.

## Version

Pinned to `1.4.3` via a single `ARG AGENT_VERSION` at image build time — it
drives the install, the `eval.agent.version` label, and `/opt/agent/VERSION`,
which the combination image reuses so it installs the same version. Immutable
per image; override at build with `build agent --agent-version <x>` per
[RULES.md](../RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
