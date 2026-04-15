# bob

IBM Bob Shell: autonomous SDLC partner agent for the terminal.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [ibm/bob-shell](https://github.com/ibm/bob-shell) |
| Version | `1.0.1` |
| Install mechanism | Direct tarball (IBM Cloud Object Storage) via `npm install -g` |
| Language runtime | Node.js 22 |

## What it does

Bob Shell is IBM's CLI SDLC agent: it plans, edits, and executes shell commands against a workspace. The Dockerfile bypasses the upstream `bobshell.sh` installer (which fetches a mutable version file from S3 on every run) and instead downloads a pinned `bobshell-1.0.1.tgz` from IBM Cloud Object Storage, then installs it globally via npm so the exact version is frozen into the image.

## How Dock runs it

The entrypoint points `OPENAI_BASE_URL` at `http://model:4000`, sets `OPENAI_API_KEY` to the proxy key, accepts the IBM license non-interactively (`BOB_ACCEPT_LICENSE=1`), disables telemetry, and runs `bob --yolo --accept-license "$TASK"`. `--yolo` auto-approves all tool calls since Docker is the sandbox. Output is printed to stdout.

## Version

Pinned to `1.0.1` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` (or the `BOB_VERSION` build arg) — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
