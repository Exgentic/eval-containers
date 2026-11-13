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
(no env expansion), so the entrypoint interpolates it at runtime. It also writes
`reply-cap.js`, a pi extension loaded with `-e` that sets every request's
output cap to `$PI_MAX_OUTPUT_TOKENS` (default 128000, as opencode's is; lower it
through the launch's agent env for a model whose window is under ~160k): pi's own default for a custom model is
16384 and pi-ai clamps any `maxTokens` in `models.json` to 32000, and a reasoning
model that spends the cap thinking ends the run with exit 0 mid-task. It then runs
`pi --no-session -e reply-cap.js -p --model eval-containers/$MODEL "$TASK"` — `-p` is
non-interactive print mode (tools run unattended, no permission prompts),
`--no-session` keeps each run self-contained, and `PI_OFFLINE=1` disables the
pi.dev update/telemetry calls that would fail in an egress-locked container.

When the launch sets `EVAL_MODEL_CONTEXT` (the model's context window, [agents rules 25–26](../../../.agents/agents/RULES.md)), the entrypoint adds it to `models.json` as `contextWindow` and caps each reply at min(`$PI_MAX_OUTPUT_TOKENS` or 128000, window/4), so every request fits the window (a 32,768-token reply on a 131k model). `settings.json` sets `reserveTokens` to that cap (never below pi's 16384), so pi's own compaction point, window − `reserveTokens` (~98k on a 131k model), fits within the window together with a reply. In print mode (`pi -p`, which this image runs) pi checks that point only when the agent run ends, not between tool calls, so it never compacts during a task. Instead it compacts when the server rejects a call as too long, then retries. pi 0.73.1 recovers like this once per task: at the second rejection it stops with no answer and exits 0. Unset, nothing changes.

## Version

Pinned to `0.73.1` via a single `ARG AGENT_VERSION` at image build time — it
drives the install, the `eval.agent.version` label, and `/opt/agent/VERSION`,
which the combination image reuses so it installs the same version. Immutable
per image; override at build with `build agent --agent-version <x>` per
[RULES.md](../RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
