# syntax=docker/dockerfile:1
# Aider agent image (AI pair programming tool)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="aider"
LABEL dock.agent.description="Aider AI pair programming tool"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/Aider-AI/aider"
LABEL dock.agent.version="0.86.2"
ENV DOCK_AGENT_VERSION_DEFAULT="0.86.2"
ENV DOCK_AGENT=aider

RUN apt-get update && apt-get install -y curl python3 python3-pip python3-venv && rm -rf /var/lib/apt/lists/*

# Install aider into a virtual environment
RUN python3 -m venv /opt/aider-venv \
    && /opt/aider-venv/bin/pip install --no-cache-dir aider-chat==0.86.2

ENV PATH="/opt/aider-venv/bin:$PATH"

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y python3 python3-pip python3-venv && rm -rf /var/lib/apt/lists/*
python3 -m venv /opt/aider-venv
/opt/aider-venv/bin/pip install --no-cache-dir aider-chat==0.86.2
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

export PATH="/opt/aider-venv/bin:$PATH"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

exec aider \
  --yes-always \
  --no-git \
  --no-auto-commits \
  --no-stream \
  --openai-api-base "${OPENAI_BASE_URL:-http://model:4000}" \
  --model "openai/${DOCK_MODEL:-default}" \
  --message "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
