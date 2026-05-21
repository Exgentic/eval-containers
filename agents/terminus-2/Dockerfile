# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)
FROM quay.io/eval-containers/core/agent-base-python:latest
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
# venv via uv's managed Python and run from it. Single install.sh is the
# source of truth for both the agent image and combo builds.
COPY run.py /opt/agent/run.py
RUN chmod +x /opt/agent/run.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
# tmux is invoked by harbor's terminal session controller at runtime.
apt-get update && apt-get install -y --no-install-recommends tmux && rm -rf /var/lib/apt/lists/*
# UV_PYTHON_INSTALL_DIR moves managed Python out of /root (mode 0700 in the
# combo image) so uid 1002 can traverse the venv's symlink chain at runtime.
export UV_PYTHON_INSTALL_DIR=/opt/uv-python
command -v uv >/dev/null 2>&1 || pip install --no-cache-dir --quiet uv
uv python install --quiet 3.12
uv venv --quiet --python 3.12 /opt/terminus2-venv
uv pip install --quiet --python /opt/terminus2-venv/bin/python harbor==0.6.4 asciinema
chmod -R a+rX /opt/uv-python /opt/terminus2-venv
I
RUN /opt/agent/install.sh
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
exec /opt/terminus2-venv/bin/python /opt/agent/run.py
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
