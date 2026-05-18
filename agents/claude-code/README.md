# claude-code

Anthropic Claude Code: Anthropic's official command-line coding agent.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [anthropics/claude-code](https://github.com/anthropics/claude-code) |
| Version | `2.1.104` |
| Install mechanism | npm (`@anthropic-ai/claude-code`) |
| Language runtime | Node.js 22 |

## What it does

Claude Code is Anthropic's terminal coding agent: it reads, edits, and runs code in the current working directory using Claude models. It talks to the Anthropic API directly; Eval Containers points it at the LiteLLM proxy via `ANTHROPIC_BASE_URL` so every call is routed through `http://model:4000`. Strength: strong tool use and file editing tuned for Claude.

## How Eval Containers runs it

The entrypoint sets `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, and `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1` (to avoid beta headers the proxy may not support), then runs `claude -p --dangerously-skip-permissions "$TASK"`. `-p` is print mode — the final answer is written to stdout. `--dangerously-skip-permissions` is safe because Docker is the sandbox.

## Version

Pinned to `2.1.104` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
