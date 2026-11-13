# crush

Charm Crush: terminal AI coding assistant from the Charm toolkit.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [charmbracelet/crush](https://github.com/charmbracelet/crush) |
| Version | `0.57.0` |
| Install mechanism | GitHub release tarball (`crush_0.57.0_Linux_*.tar.gz`) |
| Language runtime | Go |

## What it does

Crush is a TUI / headless coding agent with an integrated tool set (bash, edit, multiedit, fetch, LSP, todos, MCP). It supports OpenAI-compatible providers out of the box, which is exactly how Eval Containers wires it to the proxy. Strength: focused, ergonomic coding loops and good LSP integration.

## How Eval Containers runs it

The entrypoint writes `$XDG_CONFIG_HOME/crush/crush.json` declaring a `eval-containers` provider of type `openai-compat` whose `base_url` is `$OPENAI_BASE_URL/v1` and whose `api_key` is `$OPENAI_API_KEY`. All tools are allowlisted, metrics/notifications/auto-update are disabled. It then runs `crush run -q -m "eval-containers/$MODEL" "$TASK"` and prints the answer to stdout.

The model's `context_window` is `$EVAL_MODEL_CONTEXT` when the launch sets it ([agents rules 25–26](../../../.agents/agents/RULES.md)), and 128000 otherwise. crush keeps its own summarizing: when 20% of the window is left (20k above 200k). Its reply cap, `default_max_tokens` 8192, fits that from a 40,960-token window up; a smaller window fails before the first call.

## Version

Pinned to `0.57.0` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
