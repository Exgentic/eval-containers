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

The entrypoint writes `~/.config/opencode/opencode.json` with a `eval-containers` provider (`baseURL: $OPENAI_BASE_URL/v1`), allow-listed permissions for edit/bash/webfetch, `autoshare: false`, and both auto-update and model fetching disabled. It then runs `opencode run --model "eval-containers/$EVAL_MODEL" "$TASK"`. Dummy `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` are set to silence auth prompts.

## Version

Pinned to `1.4.3` via a single `ARG AGENT_VERSION` at image build time — it
drives the install, the `eval.agent.version` label, and `/opt/agent/VERSION`,
which the combination image reuses so it installs the same version. Immutable
per image; override at build with `build agent --agent-version <x>` per
[RULES.md](../RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
