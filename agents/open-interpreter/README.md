# open-interpreter

Open Interpreter — a natural-language code-execution agent that runs Python/shell locally from an LLM loop.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [OpenInterpreter/open-interpreter](https://github.com/OpenInterpreter/open-interpreter) |
| Version | `0.4.3` |
| Install mechanism | pip (`open-interpreter`) |
| Language runtime | Python 3.12 |

## What it does

Open Interpreter gives an LLM a persistent code-execution session — Python, shell, AppleScript, JavaScript — and a natural-language front door. The agent proposes code, the container runs it, the result loops back into the conversation. Strength: fast exploratory tasks where "write and run the snippet" beats tool-routing ceremony.

## How Dock runs it

The entrypoint sets the LiteLLM proxy endpoint and API key, invokes `interpreter` with `--auto_run --no_tts --plain` so it executes without prompting for confirmation, and captures stdout. The container is the sandbox; Docker isolates any mistakes. The final stdout line is the submission.

## Version

Pinned to `0.4.3` at image build time. Override with `DOCK_AGENT_VERSION=<version>` at build or run time per [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image (extends `core/agent-base-python`)
- `README.md` — this file
