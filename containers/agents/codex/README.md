# codex

OpenAI Codex CLI: autonomous coding agent from OpenAI.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [openai/codex](https://github.com/openai/codex) |
| Version | `0.120.0` |
| Install mechanism | npm (`@openai/codex`) |
| Language runtime | Node.js 22 |

## What it does

Codex CLI is OpenAI's terminal coding agent. It edits and executes code against the current workspace using OpenAI models (or any OpenAI-compatible endpoint configured through its `config.toml`). Eval Containers registers a `eval-containers` model provider pointing at the LiteLLM proxy so every request leaves via `http://model:4000`.

## How Eval Containers runs it

The entrypoint writes `~/.codex/config.toml` declaring a `eval-containers` model provider whose `base_url` is `$OPENAI_BASE_URL` and whose `env_key` is `OPENAI_API_KEY`, and enabling the `web_search` and `view_image` tools (so the agent can research the web and read image attachments — needed by benchmarks like GAIA). It then runs `codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "$TASK"`. Approvals are bypassed because Docker is the sandbox. Answer is streamed to stdout.

## Version

Pinned to `0.120.0` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
