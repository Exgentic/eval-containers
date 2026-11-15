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

When the launch sets `EVAL_MODEL_CONTEXT` (the model's context window, [agents rules 25–26](../../../.agents/agents/RULES.md)), the entrypoint passes it as `GOOSE_CONTEXT_LIMIT` and caps each reply (`GOOSE_MAX_TOKENS`) at min(32000, window/5), so goose's own compaction point, 80% of the window, fits together with a reply (26,214 on a 131k model). The cap replaces both the 4096 goose uses for a model it doesn't know and a catalog model's own cap. `goose run` checks the 80% point once, before the first request; during a task it compacts when the server rejects a call as too long, then retries, and stops only at a second rejection in a row. A hosted model whose output limit is below the cap rejects it with a 400 that goose reads as an overflow, so don't set `EVAL_MODEL_CONTEXT` for one. A window under 20,480 fails before the first call. Unset, goose assumes 128k and its own reply cap.

## Version

Pinned to `1.30.0` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
