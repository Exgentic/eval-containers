# mini-swe-agent

Mini-SWE-agent: lightweight SWE coding agent from the SWE-agent team.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [SWE-agent/mini-swe-agent](https://github.com/SWE-agent/mini-swe-agent) |
| Version | `2.4.6` |
| Install mechanism | pip (`mini-swe-agent`, into `/opt/mini-venv`) |
| Language runtime | Python |

## What it does

Mini-SWE-agent is the minimal sibling of the full SWE-agent: a single bash tool, a linear message history, and no custom editing commands. It reads, edits, and runs code in its working directory until it submits with `echo COMPLETE_TASK_AND_SUBMIT_FINAL_OUTPUT`. It uses LiteLLM under the hood, so any OpenAI-compatible endpoint works — including the Eval Containers gateway.

## How Eval Containers runs it

The entrypoint runs `mini --model "openai/$MODEL" --yolo --exit-immediately --cost-limit 0 --task "$TASK"` against `OPENAI_BASE_URL`. `--yolo` auto-approves actions; `--exit-immediately` skips the post-submit confirmation prompt. Mini's own cost tracking is set to ignore unknown models and its cost cap is disabled — the gateway is the cost authority and `run-agent` enforces the timeout. Its config and trajectory go to `/tmp/mini-swe-agent`. The agent's messages stream to stdout.

## Version

Pinned to `2.4.6` via `ARG AGENT_VERSION`. Override at build time with `build agent --agent-version <x>` — see [agents/RULES.md](../../../.agents/agents/RULES.md) rule 13.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
