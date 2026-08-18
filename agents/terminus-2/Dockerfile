# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.6.4
LABEL eval.type="agent"
LABEL eval.agent.name="terminus-2"
LABEL eval.agent.description="Harbor Terminus-2 reference agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/harbor-ai/harbor"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19) — Terminus-2 acts by
# typing into a terminal, so the servers are reached through the
# `mcp-bridge` command below. The label is for discovery only;
# /opt/agent/MCP is what the runtime guard reads, because labels do not
# survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=terminus-2
RUN echo true > /opt/agent/MCP
# harbor>=0.2.0 requires Python >=3.12; several benchmark bases ship 3.10
# or 3.11 (appworld 3.11, ubuntu:22.04 3.10). Provision a self-contained
# venv via the shared `/opt/agent/setup-uv-venv.sh` helper
# (core/agent-base-python).
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
# tmux is invoked by harbor's terminal session controller at runtime.
apt-get update && apt-get install -y --no-install-recommends tmux && rm -rf /var/lib/apt/lists/*
/opt/agent/setup-uv-venv.sh /opt/terminus2-venv harbor==${AGENT_VERSION} asciinema
# Terminus-2 reaches MCP servers by typing `mcp` into its terminal, so the
# bridge has to be on PATH for the agent's own commands, not just ours.
ln -sf /opt/agent/mcp-bridge /usr/local/bin/mcp
I

RUN /opt/agent/install.sh
# Late COPY: editing run.py here invalidates only the layers below,
# not the multi-minute apt+venv install above.
COPY run.py /opt/agent/run.py
COPY mcp-bridge /opt/agent/mcp-bridge
RUN chmod +x /opt/agent/run.py /opt/agent/mcp-bridge
RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
exec /opt/terminus2-venv/bin/python /opt/agent/run.py
E

ENTRYPOINT ["/run.sh"]
