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

Goose is Block's extensible AI agent that uses pluggable "extensions" (filesystem, shell, developer tools, MCP) to act on a workspace. It speaks to any OpenAI-compatible provider via the `openai` provider selector. Dock binds that to the LiteLLM proxy through `OPENAI_HOST`.

## How Dock runs it

The entrypoint sets `GOOSE_PROVIDER=openai`, `GOOSE_MODEL=$DOCK_MODEL`, `OPENAI_API_KEY`, and `OPENAI_HOST=$OPENAI_BASE_URL`, disables telemetry, and runs `goose run -t "$TASK" --no-session -q`. `--no-session` keeps evaluation runs one-shot, `-q` suppresses the banner so only the model response reaches stdout.

## Version

Pinned to `1.30.0` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
