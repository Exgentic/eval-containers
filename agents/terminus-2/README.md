# terminus-2

Harbor Terminus-2: the reference terminal-driving agent from Harbor Framework.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [harbor-ai/harbor](https://github.com/harbor-ai/harbor) |
| Version | `0.3.0` |
| Install mechanism | pip (`harbor`, which ships Terminus-2) |
| Language runtime | Python 3.12 |

## What it does

Terminus-2 is Harbor's reference coding agent. It has no standalone CLI — it's an out-of-container orchestrator that drives a sandboxed environment via tmux through Harbor's `BaseEnvironment` SDK. Dock's image installs `harbor==0.3.0` and ships a custom `/opt/agent/run.py` that provides a `LocalEnvironmentShim` so Terminus-2 can execute commands inside this container directly.

## How Dock runs it

The entrypoint sets `OPENAI_API_KEY` / `OPENAI_BASE_URL` and execs `python3 /opt/agent/run.py`. The wrapper imports `harbor.agents.terminus_2.Terminus2`, constructs it with `api_base=$OPENAI_BASE_URL`, runs `await agent.run(instruction=$TASK, environment=shim, context=AgentContext())`, then walks `_trajectory_steps` in reverse to print the last assistant message to stdout. Trial and log dirs are created under `tempfile.mkdtemp`.

## Version

Pinned to `0.3.0` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image (also inlines `/opt/agent/run.py` wrapper and `/opt/agent/install.sh`)
- `README.md` — this file
