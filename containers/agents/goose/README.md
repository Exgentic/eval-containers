# goose

Block Goose: open-source AI agent framework from Block.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [block/goose](https://github.com/block/goose) |
| Version | `1.30.0` |
| Install mechanism | GitHub release binary (`goose-<arch>-unknown-linux-gnu.tar.bz2`) |
| Language runtime | Rust |

## What it does

Goose is Block's extensible AI agent that uses pluggable "extensions" (filesystem, shell, developer tools, MCP) to act on a workspace. It speaks to any OpenAI-compatible provider via the `openai` provider selector. Eval Containers binds that to the LiteLLM proxy through `OPENAI_HOST`.

## How Eval Containers runs it

The entrypoint sets `GOOSE_PROVIDER=openai`, `GOOSE_MODEL=$EVAL_MODEL`, `OPENAI_API_KEY`, and `OPENAI_HOST=$OPENAI_BASE_URL`, disables telemetry, and runs `goose run -t "$TASK" --no-session -q`. `--no-session` keeps evaluation runs one-shot, `-q` suppresses the banner so only the model response reaches stdout.

When the launch sets `EVAL_MODEL_CONTEXT` (the model's context window, [agents rules 25–26](../../../.agents/agents/RULES.md)), the entrypoint passes it as `GOOSE_CONTEXT_LIMIT`, and goose keeps its own compaction at 80% of it (on a 131k model, at ~105k). goose caps a reply at 4096 tokens for a model it doesn't know; a model in goose's own catalog keeps that catalog's cap. A window under 20,480 fails before the first call. Unset, goose assumes 128k.

## Version

Pinned to `1.30.0` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
