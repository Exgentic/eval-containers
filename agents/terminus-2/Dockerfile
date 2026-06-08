# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="terminus-2"
LABEL eval.agent.description="Harbor Terminus-2 reference agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/harbor-ai/harbor"
LABEL eval.agent.version="0.6.4"
ENV EVAL_AGENT_VERSION_DEFAULT="0.6.4"
ENV EVAL_AGENT=terminus-2
# harbor>=0.2.0 requires Python >=3.12; several benchmark bases ship 3.10
# or 3.11 (appworld 3.11, ubuntu:22.04 3.10). Provision a self-contained
# venv via the shared `/opt/agent/setup-uv-venv.sh` helper
# (core/agent-base-python).
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
# tmux is invoked by harbor's terminal session controller at runtime.
apt-get update && apt-get install -y --no-install-recommends tmux && rm -rf /var/lib/apt/lists/*
/opt/agent/setup-uv-venv.sh /opt/terminus2-venv harbor==0.6.4 asciinema
I

RUN /opt/agent/install.sh
# Late COPY: editing run.py here invalidates only the layers below,
# not the multi-minute apt+venv install above.
COPY run.py /opt/agent/run.py
RUN chmod +x /opt/agent/run.py
RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
exec /opt/terminus2-venv/bin/python /opt/agent/run.py
E

ENTRYPOINT ["/run.sh"]
