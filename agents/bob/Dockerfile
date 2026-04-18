# syntax=docker/dockerfile:1
# IBM Bob Shell agent image
FROM quay.io/dock-eval/core/agent-base-node:latest
LABEL dock.type="agent"
LABEL dock.agent.name="bob"
LABEL dock.agent.description="IBM Bob Shell — AI SDLC partner"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/ibm/bob-shell"
LABEL dock.agent.version="1.0.1"
ENV DOCK_AGENT_VERSION_DEFAULT="1.0.1"
ENV DOCK_AGENT=bob
ARG BOB_VERSION=1.0.1
RUN curl -fsSL "https://s3.us-south.cloud-object-storage.appdomain.cloud/bob-shell/bobshell-${BOB_VERSION}.tgz" -o /tmp/bob.tgz && \
    npm install -g /tmp/bob.tgz && rm /tmp/bob.tgz
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx bob; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export BOB_ACCEPT_LICENSE=1
export BOB_TELEMETRY=off
exec bob --yolo --accept-license "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
