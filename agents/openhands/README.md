# openhands

OpenHands: open-source autonomous AI software engineer from All-Hands-AI.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [All-Hands-AI/OpenHands](https://github.com/All-Hands-AI/OpenHands) |
| Version | `1.14.0` |
| Install mechanism | GitHub release binary (`openhands-linux-x86_64`, linux/amd64 only) |
| Language runtime | Python (prebuilt binary on `python:3.12-slim`) |

## What it does

OpenHands (formerly OpenDevin) is an autonomous software engineer that plans, edits, and runs code across an entire workspace. The Dockerfile intentionally skips the upstream `install.sh` (which hard-codes the version inside an uncontrolled script) and curls the pinned `openhands-linux-x86_64` release asset directly so builds don't drift.

## How Dock runs it

The entrypoint sets `LLM_BASE_URL=$OPENAI_BASE_URL`, `LLM_API_KEY=$OPENAI_API_KEY`, `LLM_MODEL=$DOCK_MODEL`, then runs `openhands --headless --always-approve --override-with-envs -t "$TASK"`. `--headless` gives non-interactive execution, `--always-approve` auto-approves commands, `--override-with-envs` forces the LLM env vars over any bundled config. Answer goes to stdout.

## Version

Pinned to `1.14.0` at image build time via the `OPENHANDS_VERSION` build arg. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
