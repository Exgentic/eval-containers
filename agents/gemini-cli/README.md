# gemini-cli

Google Gemini CLI: Google's terminal coding agent for Gemini models.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) |
| Version | `0.37.2` |
| Install mechanism | npm (`@google/gemini-cli`) |
| Language runtime | Node.js 22 |

## What it does

Gemini CLI is Google's official terminal agent for the Gemini model family. It supports file editing, shell execution, and multi-step planning. Eval Containers redirects the Gemini endpoint to the LiteLLM proxy via `GOOGLE_GEMINI_BASE_URL`, so the proxy sees and rewrites every call.

## How Eval Containers runs it

The entrypoint sets `GOOGLE_GEMINI_BASE_URL`, `GEMINI_API_KEY`, and `GEMINI_SANDBOX=false` (Docker is the sandbox), then runs `gemini --yolo -p "$TASK"`. `--yolo` auto-approves tool calls, `-p` runs the CLI in headless / print mode so the final answer goes to stdout.

## Version

Pinned to `0.37.2` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
