# syntax=docker/dockerfile:1
# GitHub Copilot CLI agent image

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="copilot-cli"
LABEL dock.agent.description="GitHub Copilot CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/github/copilot-cli"
ENV DOCK_AGENT=copilot-cli

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install GitHub Copilot CLI (pinned version)
RUN npm install -g @github/copilot@1.0.24

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @github/copilot@1.0.24
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Route all LLM calls through the Dock proxy (BYOK / offline mode)
export COPILOT_PROVIDER_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}/v1"
export COPILOT_PROVIDER_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export COPILOT_OFFLINE=true

# Use whichever model the proxy advertises
export COPILOT_MODEL="${COPILOT_MODEL:-default}"

# Run non-interactively: pipe task via stdin, --yolo auto-approves all tools
# Docker is the sandbox so full tool access is safe
echo "$TASK" | exec copilot --yolo
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
