# syntax=docker/dockerfile:1
# Codex CLI agent image (OpenAI)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="codex"
ENV DOCK_AGENT=codex
LABEL dock.agent.description="OpenAI Codex CLI"
LABEL dock.agent.runtime="node"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Codex CLI
RUN npm install -g @openai/codex

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @openai/codex
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
exec timeout ${DOCK_TIMEOUT:-300} codex --approval-mode full-auto "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
