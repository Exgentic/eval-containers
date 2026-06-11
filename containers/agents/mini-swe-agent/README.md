# mini-swe-agent

Mini-SWE-agent: lightweight SWE coding agent from the Princeton SWE-agent team.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [princeton-nlp/SWE-agent](https://github.com/princeton-nlp/SWE-agent) |
| Version | `2.2.8` |
| Install mechanism | pip (`mini-swe-agent`, into `/opt/swe-agent-venv`) |
| Language runtime | Python |

## What it does

Mini-SWE-agent is the stripped-down, single-binary-ish version of the full SWE-agent: a software-engineering agent that reads, edits, and runs code against a local workspace. It uses LiteLLM under the hood, so any OpenAI-compatible endpoint works — including the Eval Containers proxy.

## How Eval Containers runs it

The entrypoint activates the venv, sets `OPENAI_API_KEY` and `OPENAI_BASE_URL` (pointing at `http://model:4000`), and runs `mini --model "openai/$EVAL_MODEL" --yolo --task "$TASK"`. `--yolo` auto-approves actions. The agent prints its answer and trajectory to stdout.

## Version

Pinned to `2.2.8` at image build time. Override with `EVAL_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
