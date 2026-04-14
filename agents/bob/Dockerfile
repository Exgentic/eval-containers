# syntax=docker/dockerfile:1
# IBM Bob Shell agent image

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="bob"
LABEL dock.agent.description="IBM Bob Shell — AI SDLC partner"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/ibm/bob-shell"
LABEL dock.agent.version="1.0.1"
ENV DOCK_AGENT=bob

ARG BOB_VERSION=1.0.1

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x (Bob Shell requires >= 22.15)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install IBM Bob Shell — pinned version, direct tarball download.
# The upstream install script (bob.ibm.com/download/bobshell.sh) fetches a
# MUTABLE version file from S3 on every run, so every build would drift.
# We bypass it and install $BOB_VERSION directly via npm, which pins the
# exact version into the image. Bump $BOB_VERSION in the ARG above to upgrade.
RUN curl -fsSL "https://s3.us-south.cloud-object-storage.appdomain.cloud/bob-shell/bobshell-${BOB_VERSION}.tgz" \
      -o /tmp/bobshell.tgz \
    && npm install -g /tmp/bobshell.tgz \
    && rm /tmp/bobshell.tgz

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
: "${BOB_VERSION:=1.0.1}"
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*
curl -fsSL "https://s3.us-south.cloud-object-storage.appdomain.cloud/bob-shell/bobshell-${BOB_VERSION}.tgz" -o /tmp/bobshell.tgz
npm install -g /tmp/bobshell.tgz
rm /tmp/bobshell.tgz
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
