# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="openhands"
LABEL dock.agent.description="OpenHands open-source AI software engineer"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/All-Hands-AI/OpenHands"
ENV DOCK_AGENT=openhands

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Install OpenHands CLI
RUN curl -fsSL https://install.openhands.dev/install.sh | sh

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*
curl -fsSL https://install.openhands.dev/install.sh | sh
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

export LLM_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export LLM_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export LLM_MODEL="${DOCK_MODEL:-gpt-4o}"

exec openhands --headless --always-approve --override-with-envs -t "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
