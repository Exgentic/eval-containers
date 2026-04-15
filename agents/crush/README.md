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

Crush is a TUI / headless coding agent with an integrated tool set (bash, edit, multiedit, fetch, LSP, todos, MCP). It supports OpenAI-compatible providers out of the box, which is exactly how Dock wires it to the proxy. Strength: focused, ergonomic coding loops and good LSP integration.

## How Dock runs it

The entrypoint writes `$XDG_CONFIG_HOME/crush/crush.json` declaring a `dock` provider of type `openai-compat` whose `base_url` is `$OPENAI_BASE_URL/v1` and whose `api_key` is `$OPENAI_API_KEY`. All tools are allowlisted, metrics/notifications/auto-update are disabled. It then runs `crush run -q -m "dock/$DOCK_MODEL" "$TASK"` and prints the answer to stdout.

## Version

Pinned to `0.57.0` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
