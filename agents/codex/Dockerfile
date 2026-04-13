# syntax=docker/dockerfile:1
# Codex CLI agent image (OpenAI)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="codex"
ENV DOCK_AGENT=codex
LABEL dock.agent.description="OpenAI Codex CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/openai/codex"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Codex CLI
RUN npm install -g @openai/codex@0.120.0

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @openai/codex@0.120.0
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
mkdir -p ~/.codex
cat > ~/.codex/config.toml <<CONF
model = "${CODEX_MODEL:-default}"
model_provider = "dock"

[model_providers.dock]
name = "Dock LiteLLM Proxy"
base_url = "${OPENAI_BASE_URL:-http://model:4000}/v1"
env_key = "OPENAI_API_KEY"
CONF

export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
