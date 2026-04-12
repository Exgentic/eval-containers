# syntax=docker/dockerfile:1
# Claude Code agent image
# Standalone image with runtime. Primary purpose: provide /opt/agent/ for combination.

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="claude-code"
LABEL dock.agent.description="Anthropic Claude Code CLI"
LABEL dock.agent.runtime="bun"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Bun (Claude Code runtime)
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:$PATH"

# Install Claude Code
RUN npm install -g @anthropic-ai/claude-code

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://bun.sh/install | bash
export PATH="/root/.bun/bin:$PATH"
npm install -g @anthropic-ai/claude-code
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
exec timeout ${DOCK_TIMEOUT:-300} claude -p --dangerously-skip-permissions "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
