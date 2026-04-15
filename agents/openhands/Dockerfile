# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="openhands"
LABEL dock.agent.description="OpenHands open-source AI software engineer"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL dock.agent.version="1.14.0"
ENV DOCK_AGENT_VERSION_DEFAULT="1.14.0"
ENV DOCK_AGENT=openhands

ARG OPENHANDS_VERSION=1.14.0

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Install OpenHands CLI — direct binary from GitHub releases, pinned.
# We skip the upstream install.sh because it hardcodes the version inside
# a script we don't control — every bump to that script would silently
# change our build. Pulling the release asset directly pins it here.
# Project targets linux/amd64 end-to-end, so x86_64 is hardcoded.
RUN curl -fsSL -o /usr/local/bin/openhands \
      "https://github.com/OpenHands/OpenHands-CLI/releases/download/${OPENHANDS_VERSION}/openhands-linux-x86_64" \
    && chmod +x /usr/local/bin/openhands

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
: "${OPENHANDS_VERSION:=1.14.0}"
apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*
curl -fsSL -o /usr/local/bin/openhands \
  "https://github.com/OpenHands/OpenHands-CLI/releases/download/${OPENHANDS_VERSION}/openhands-linux-x86_64"
chmod +x /usr/local/bin/openhands
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
