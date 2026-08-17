# factory

Factory Droid CLI: autonomous software-engineering agent for the terminal.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [docs.factory.ai](https://docs.factory.ai/droid-cli/getting-started/overview) |
| Version | `0.197.0` |
| Install mechanism | `npm install -g droid@<version>` |
| Language runtime | Node.js 22 |
| Smoke test | **blocked** — see [broken.md](../../../tests/run/agents/broken.md) |

## What it does

Droid plans and executes shell commands and file edits against a workspace.
`droid exec` is its headless mode: one prompt in, an answer on stdout.

## How Eval Containers runs it

Droid ignores `OPENAI_BASE_URL`. The gateway is reachable only through a BYOK
`customModels` entry, so the entrypoint writes `$HOME/.factory/settings.json`
with `provider: "generic-chat-completion-api"` pointed at `$OPENAI_BASE_URL`,
keyed by the model id passed to `-m`. `--skip-permissions-unsafe` auto-approves
tool calls because Docker is the sandbox (rule 10); `-o text` puts the answer on
stdout (rule 3).

The npm distribution disables droid's self-updater, and the image sets
`FACTORY_DROID_AUTO_UPDATE_ENABLED=false` as a belt-and-braces guard so the pin
holds at runtime.

## Known limitation: a Factory account is required

Droid authenticates to Factory **before** it will use a BYOK model, so this
agent cannot run — or be smoke-tested — without a `FACTORY_API_KEY`. Measured
on `droid@0.197.0`:

```
# no key
Authentication failed. Please log in using /login or set a valid
FACTORY_API_KEY environment variable.

# invalid key
Authentication failed. FACTORY_API_KEY is set but appears to be invalid.
```

The second message appears identically with `--network=none`, so the check is
local format validation rather than a call to Factory — but it gates the run
either way. This is a vendor account requirement, not a bug, and the image is
otherwise contract-complete: supply a real `FACTORY_API_KEY` and it runs.

Note this also means the agent reaches Factory's servers for auth on a normal
run, which is in tension with rule 9 (no agent internet). A benchmark using this
agent must allow that egress.

## Version

Pinned to `0.197.0` at image build time. Override with the `AGENT_VERSION` build
arg — see [RULES.md](../RULES.md) principles 12 and 13.

## Files

- `Dockerfile` — builds the agent image
- `docker-bake.hcl` — bake target
- `README.md` — this file
