# syntax=docker/dockerfile:1
# Gemini CLI agent image (Google)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="gemini-cli"
LABEL dock.agent.description="Google Gemini CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/google-gemini/gemini-cli"
LABEL dock.agent.version="0.37.2"
ENV DOCK_AGENT=gemini-cli

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Gemini CLI (pinned version)
RUN npm install -g @google/gemini-cli@0.37.2

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @google/gemini-cli@0.37.2
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Route all LLM calls through the Dock proxy
export GOOGLE_GEMINI_BASE_URL="${GOOGLE_GEMINI_BASE_URL:-http://model:4000}"
export GEMINI_API_KEY="${GEMINI_API_KEY:-sk-proxy}"

# Disable sandbox — Docker is the sandbox
export GEMINI_SANDBOX=false

# Run non-interactively: --yolo (auto-approve tool use), -p (headless mode with prompt)
exec gemini --yolo -p "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
