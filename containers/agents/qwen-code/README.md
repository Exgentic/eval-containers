# qwen-code

Qwen Code: Alibaba's terminal coding agent (fork of gemini-cli tuned for Qwen models).

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [QwenLM/qwen-code](https://github.com/QwenLM/qwen-code) |
| Version | `0.14.4` |
| Install mechanism | npm (`@qwen-code/qwen-code`) |
| Language runtime | Node.js 22 |

## What it does

Qwen Code is a fork of Google's gemini-cli, re-tuned and re-branded by Alibaba for the Qwen model family. It keeps gemini-cli's tooling (file edit, shell, plan/print modes) and adds an `openai` `modelProviders` block that targets any OpenAI-compatible endpoint — which Eval Containers points at the LiteLLM proxy.

## How Eval Containers runs it

The entrypoint writes `~/.qwen/settings.json` declaring an `openai` model provider whose `baseUrl` is `$OPENAI_BASE_URL/v1`, sets `OPENAI_API_KEY` / `OPENAI_BASE_URL` / `OPENAI_MODEL`, disables both Gemini and Qwen sandboxes (Docker is the sandbox), and runs `qwen --yolo -p "$TASK"` for a headless, auto-approved run. Answer goes to stdout.

## Version

Pinned to `0.14.4` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
