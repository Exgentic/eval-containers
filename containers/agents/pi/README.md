# pi

`pi` — a minimal, terminal-native coding agent with built-in read/bash/edit/write tools (Mario Zechner / earendil.works).

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [mariozechner/pi-mono](https://github.com/mariozechner/pi-mono) |
| Version | `0.73.1` |
| Install mechanism | npm (`@mariozechner/pi-coding-agent`) |
| Language runtime | Node.js 22 |

## What it does

`pi` runs a compact agent loop over a small set of built-in tools (read, bash,
edit, write). It speaks several provider APIs directly and adds custom
providers through a `models.json` config. Eval Containers registers a single
custom provider that points every request at the gateway's OpenAI-compatible
endpoint, so all model traffic routes through one logged proxy.

## How Eval Containers runs it

The entrypoint writes `models.json` under `PI_CODING_AGENT_DIR`, declaring one
provider (`eval-containers`) whose `baseUrl` is `$OPENAI_BASE_URL` and whose
`api` is `openai-completions`. `models.json` takes the base URL as a literal
(no env expansion), so the entrypoint interpolates it at runtime. It then runs
`pi --no-session -p --model eval-containers/$EVAL_MODEL "$TASK"` — `-p` is
non-interactive print mode (tools run unattended, no permission prompts),
`--no-session` keeps each run self-contained, and `PI_OFFLINE=1` disables the
pi.dev update/telemetry calls that would fail in an egress-locked container.

## Version

Pinned to `0.73.1` via a single `ARG AGENT_VERSION` at image build time — it
drives the install, the `eval.agent.version` label, and `/opt/agent/VERSION`,
which the combination image reuses so it installs the same version. Immutable
per image; override at build with `build agent --agent-version <x>` per
[RULES.md](../RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
