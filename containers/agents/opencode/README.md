# opencode

SST opencode: MIT open-source AI coding agent.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [sst/opencode](https://github.com/sst/opencode) |
| Version | `2.4.1` |
| Install mechanism | npm (`@opencode/cli`, a native binary) |
| Language runtime | native binary (installed with npm on Node.js 22) |

## What it does

opencode is SST's open-source terminal coding agent. It supports arbitrary providers declared in `opencode.json` through the Vercel AI SDK; Eval Containers uses `@ai-sdk/openai-compatible` to register the LiteLLM proxy as a `eval-containers` provider and disables the built-in `anthropic`/`openai` providers so nothing escapes the proxy.

## How Eval Containers runs it

The entrypoint writes `~/.config/opencode/opencode.json` with:

- an `eval-containers` provider (`baseURL: $OPENAI_BASE_URL`, the gateway's OpenAI endpoint);
- allow-listed permissions for edit/bash/webfetch, with `websearch` denied (with no search provider configured it asks the user to pick one, which fails an unattended run);
- MCP tools kept direct (`codemode: false`);
- auto-update and model fetching disabled.

It then runs `opencode run --standalone --model "eval-containers/$MODEL" -- "$TASK" </dev/null`. `--standalone` gives the run a private server that ends with it, never a shared background service that may already be running. Dummy `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` are set to silence auth prompts.

**Shell commands are bounded at 600 s.** opencode 2 moves a command that runs past its timeout (120 s unless the model sets one) to the background with no limit, and `opencode run` waits for every background job before it exits, so a command that never returns would hold the task until `EVAL_TIMEOUT` (#785). The image sets opencode's `shell` to a wrapper that runs each command under `timeout`, which ends its process group at 600 s; the model sees exit code 124 (137 if the command ignored SIGTERM). Compared with 1.x: 1.x killed a command at 120 s unless the model set a timeout, so one that needs 120–600 s now finishes; a longer timeout the model asks for, which 1.x honoured, is cut at 600 s, and so is a command started with `background: true`. After its final answer opencode waits for the background jobs it started, then gives the model one more turn on their results, which can add text to stdout; with the wrapper that wait is at most 600 s.

**Errors.** opencode does not recognise LiteLLM's spent-budget wording as a permanent error, so it retries that 429 like any 429 or 5xx: 10 retries, about 84 s of backoff without a `Retry-After` (a `Retry-After` is honoured, up to 15 min a retry), then exit 1, which the runner records as errored. That ends a spent budget, which 1.x retried forever, but also a throttle that outlasts the retries, which 1.x waited out. A call rejected as too long makes opencode compact the conversation and go on, and the run then exits 0; a second rejection in the same step, or a compaction that fails, exits 1. A reply cut at the output cap is continued twice; a third cut in a row ends the run with exit 1 (1.x exited 0).

opencode's reply cap is raised from its own 32000 to `$OPENCODE_MAX_OUTPUT_TOKENS` (128000 unless set), as the model's `limit.output`, because a reply cut at the cap three times running ends the run. When the launch sets `EVAL_MODEL_CONTEXT_WINDOW` ([agents rules 25–26](../../../.agents/agents/RULES.md)), the cap becomes min(that, window/4) and the window is the model's `limit.context`, which opencode compacts by. Told a window, opencode compacts ahead of it, which 1.x never did. Unset, `limit.context` is 0, a window opencode does not know: it compacts only when the provider rejects a call as too long. The derivation is in the `/run.sh` comment in the [Dockerfile](Dockerfile).

The x64 image ships opencode's baseline build, which runs on any x64 CPU; the package would otherwise pick the build for the machine that built the image.

## Version

Pinned to `2.4.1` via a single `ARG AGENT_VERSION` at image build time — it
drives the install, the `eval.agent.version` label, and `/opt/agent/VERSION`,
which the combination image reuses so it installs the same version. Immutable
per image; override at build with `build agent --agent-version <x>` per
[RULES.md](../RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
