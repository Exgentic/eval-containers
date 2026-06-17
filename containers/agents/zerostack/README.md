# zerostack

Zerostack: a minimal, memory-frugal coding agent written in Rust (gi-dellav/zerostack).

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [gi-dellav/zerostack](https://github.com/gi-dellav/zerostack) |
| Version | `1.5.0` |
| Install mechanism | GitHub release binary (`zerostack-<arch>-unknown-linux-gnu.tar.gz`) |
| Language runtime | Rust |
| Protocol | OpenAI (Chat Completions) |

## What it does

Zerostack is a single-binary terminal coding agent (≈26 MB binary, ≈16 MB RAM) with file/bash/search tools, a permission system, and an agent loop. It talks to any OpenAI-compatible endpoint through a user-defined "custom provider", which Eval Containers points at the gateway.

## How Eval Containers runs it

The entrypoint writes a custom provider to `$ZS_CONFIG_DIR/config.json` (`provider_type: openai`, `api_style: completions`) whose `base_url` is `$OPENAI_BASE_URL`, sets `OPENAI_API_KEY`, disables the default-on Exa MCP (the gateway is the only allowed egress), then runs:

    zerostack -p --provider eval --model "$MODEL" --no-session --dangerously-skip-permissions --no-context-files -- "$TASK"

`-p` prints the final answer to stdout and exits (no TUI); `--no-session` keeps the run ephemeral; `--dangerously-skip-permissions` lets the agent act unguarded because the container — not the agent — is the sandbox ([RULES.md](../RULES.md) rule 10); `--no-context-files` skips the `AGENTS.md`/`ARCHITECTURE.md` scan, which otherwise prompts `Create one? [y/N]` and pollutes stdout when there is no TTY. rig appends `/chat/completions` to the `/v1` base, so `OPENAI_BASE_URL` is passed through unmodified (rule 5).

## Note on streaming (replay fixtures)

Zerostack streams completions (it expects an SSE `text/event-stream` response). The live gateway supports streaming, so normal runs and the smoke test (which only needs the agent to *reach* the gateway) work. The `models/replay` mock returns a non-streaming `application/json` body, so a recorded replay fixture ([RULES.md](../RULES.md) rule 17) will only play back cleanly once the replay mock serves SSE for this agent — there is no non-streaming flag in the upstream CLI to fall back to.

## Version

Pinned to `1.5.0` at image build time via `ARG AGENT_VERSION`, recorded in the `eval.agent.version` label and in `/opt/agent/VERSION` (which the combination image reuses, [RULES.md](../RULES.md) rule 13). The pin is immutable per image; build a different image tag to change it.

## Files

- `Dockerfile` — builds the agent image
- `docker-bake.hcl` — build graph (extends `core/agent-base-rust`)
- `README.md` — this file
