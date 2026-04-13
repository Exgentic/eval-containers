# syntax=docker/dockerfile:1
# IBM Bob Shell agent image

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="bob"
LABEL dock.agent.description="IBM Bob Shell — AI SDLC partner"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/ibm/bob-shell"
ENV DOCK_AGENT=bob

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x (Bob Shell requires >= 22.15)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install IBM Bob Shell (pinned version)
# The official install script fetches a versioned tgz and runs npm install -g.
# We invoke it non-interactively; it auto-detects npm.
RUN curl -fsSL https://bob.ibm.com/download/bobshell.sh | bash

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
curl -fsSL https://bob.ibm.com/download/bobshell.sh | bash
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Route all LLM calls through the Dock proxy
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# Accept IBM license non-interactively
export BOB_ACCEPT_LICENSE=1

# Disable telemetry in CI/eval context
export BOB_TELEMETRY=off

# Run non-interactively: positional query triggers one-shot mode
# --yolo auto-approves all tool calls (Docker is the sandbox — Rule 10)
# --accept-license avoids interactive license prompt
exec bob --yolo --accept-license "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
