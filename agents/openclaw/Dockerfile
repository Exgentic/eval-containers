# syntax=docker/dockerfile:1
# OpenClaw agent image
# General-purpose AI agent (https://github.com/openclaw/openclaw)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="openclaw"
LABEL dock.agent.description="OpenClaw general-purpose AI agent"
LABEL dock.agent.runtime="node"
ENV DOCK_AGENT=openclaw

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install OpenClaw CLI (pinned version)
RUN npm install -g openclaw@2026.4.11

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g openclaw@2026.4.11
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Configure OpenClaw to route LLM calls through the Dock proxy
# Write config directly — avoids interactive onboarding
mkdir -p ~/.openclaw
cat > ~/.openclaw/openclaw.json <<CONF
{
  "models": {
    "default": "dock-model",
    "providers": {
      "dock": {
        "baseUrl": "${OPENAI_BASE_URL:-http://model:4000}",
        "apiKey": "${OPENAI_API_KEY:-sk-proxy}",
        "api": "openai-completions",
        "models": [{"id": "dock-model", "name": "Dock Proxy Model"}]
      }
    }
  },
  "exec": {
    "security": "full"
  }
}
CONF

export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# Run one agent turn locally (no gateway needed), plain output to stdout
exec openclaw agent --local --message "$TASK" --plain
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
