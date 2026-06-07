# Adding an Agent

Read `RULES.md` first. Then copy the template below and fill in the blanks.

## Dockerfile

```dockerfile
# {NAME} agent
FROM ubuntu:24.04

LABEL eval.type="agent"
LABEL eval.agent.name="{name}"
LABEL eval.agent.description="{Short description}"
LABEL eval.agent.runtime="{runtime}"
ENV EVAL_AGENT={name}

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install agent runtime
{INSTALL_COMMANDS}

RUN mkdir -p /opt/agent

# install.sh — runs inside the benchmark image at build time
RUN cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
{INSTALL_COMMANDS_REPEATED}
INSTALL
RUN chmod +x /opt/agent/install.sh

# entrypoint.sh — runs the agent at evaluation time
RUN cat > /agent/run.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# If the agent needs a config file for proxy routing, create it here:
# mkdir -p ~/.config/my-agent
# cat > ~/.config/my-agent/config.toml <<CONF
# base_url = "${OPENAI_BASE_URL:-http://model:4000}"
# api_key = "${OPENAI_API_KEY:-sk-proxy}"
# CONF

# Set dummy API key if the SDK requires one
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# Run the agent — print answer to stdout
exec {AGENT_COMMAND} "$TASK"
ENTRY
RUN chmod +x /agent/run.sh

ENTRYPOINT ["/agent/run.sh"]
```

## Blanks to fill

| Placeholder | Example | Description |
|-------------|---------|-------------|
| `{NAME}` | `My Agent` | Display name |
| `{name}` | `my-agent` | Lowercase with hyphens |
| `{runtime}` | `python`, `node`, `bash` | Primary runtime |
| `{INSTALL_COMMANDS}` | `pip install my-agent==1.2.3` | Commands to install the agent |
| `{INSTALL_COMMANDS_REPEATED}` | Same as above | Repeated in install.sh (runs on benchmark base) |
| `{AGENT_COMMAND}` | `my-agent run --auto` | Command that reads $TASK and runs |

## Gotchas

- **install.sh runs on the benchmark base image**, not yours. It might be `python:3.12-slim` or `ubuntu:24.04` or an upstream image. Handle missing packages.
- **Pin exact versions.** `pip install my-agent==1.2.3`, not `pip install my-agent`.
- **The agent runs as non-root `agent` user.** Don't assume root access.
- **All LLM calls must go through the proxy.** The agent receives `OPENAI_BASE_URL=http://model:4000`. If your SDK ignores this env var, write a config file in the entrypoint that points to it.
- **No self-sandboxing.** Don't use bubblewrap, seccomp, or internal sandboxes. Docker is the sandbox.
- **Print the answer to stdout.** The entrypoint captures it. Don't write to files.
- **Don't add your own timeout.** `EVAL_TIMEOUT` is enforced by the shared entrypoint.
- **Log actions to stderr.** The entrypoint captures stderr to `/output/agent/stderr.log`. Agents that log their steps (commands run, files edited) there give users visibility into what happened. This is optional but helpful.
- See `agents/raw/Dockerfile` for the simplest possible agent, `agents/codex/Dockerfile` for an agent that needs proxy config.
