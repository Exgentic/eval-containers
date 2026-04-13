# syntax=docker/dockerfile:1
# Claude Code agent image (Anthropic)
# Standalone image with runtime. Primary purpose: provide /opt/agent/ for combination.

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="claude-code"
LABEL dock.agent.description="Anthropic Claude Code CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/anthropics/claude-code"
ENV DOCK_AGENT=claude-code

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Claude Code CLI (pinned version)
RUN npm install -g @anthropic-ai/claude-code@2.1.104

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @anthropic-ai/claude-code@2.1.104
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Route all LLM calls through the Dock proxy
export ANTHROPIC_BASE_URL="${ANTHROPIC_BASE_URL:-http://model:4000}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"

# Disable experimental beta headers that the proxy may not support
export CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1

# Run non-interactively: -p (print mode) prints the answer to stdout
# --dangerously-skip-permissions: skip all permission prompts (Docker is the sandbox)
exec claude -p --dangerously-skip-permissions "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
