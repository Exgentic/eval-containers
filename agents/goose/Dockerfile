# syntax=docker/dockerfile:1
# Goose agent image (Block)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="goose"
LABEL dock.agent.description="Block Goose AI agent"
LABEL dock.agent.runtime="rust"
LABEL dock.agent.url="https://github.com/block/goose"
ENV DOCK_AGENT=goose

RUN apt-get update && apt-get install -y curl bzip2 libgomp1 && rm -rf /var/lib/apt/lists/*

# Install Goose CLI v1.30.0 — download binary directly to avoid interactive configure
RUN ARCH=$(uname -m) && \
    curl -fsSL "https://github.com/block/goose/releases/download/v1.30.0/goose-${ARCH}-unknown-linux-gnu.tar.bz2" \
    | tar xj -C /usr/local/bin --strip-components=0 ./goose \
    && chmod +x /usr/local/bin/goose

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y curl bzip2 libgomp1 && rm -rf /var/lib/apt/lists/*
ARCH=$(uname -m)
curl -fsSL "https://github.com/block/goose/releases/download/v1.30.0/goose-${ARCH}-unknown-linux-gnu.tar.bz2" \
    | tar xj -C /usr/local/bin --strip-components=0 ./goose \
    && chmod +x /usr/local/bin/goose
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Configure Goose to use the proxy as an OpenAI-compatible provider
export GOOSE_PROVIDER="openai"
export GOOSE_MODEL="${DOCK_MODEL:-default}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_HOST="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_PATH="v1/chat/completions"

# Disable telemetry
export GOOSE_TELEMETRY_OFF=true

# Run the agent non-interactively, print answer to stdout
exec goose run -t "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
