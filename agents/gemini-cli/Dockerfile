# syntax=docker/dockerfile:1
# Gemini CLI agent image (Google)
FROM quay.io/dock-eval/core/agent-base-node:latest
LABEL dock.type="agent"
LABEL dock.agent.name="gemini-cli"
LABEL dock.agent.description="Google Gemini CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/google-gemini/gemini-cli"
LABEL dock.agent.version="0.37.2"
ENV DOCK_AGENT_VERSION_DEFAULT="0.37.2"
ENV DOCK_AGENT=gemini-cli
RUN npm install -g @google/gemini-cli@0.37.2
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx gemini; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export GOOGLE_GEMINI_BASE_URL="${GOOGLE_GEMINI_BASE_URL:-http://model:4000}"
export GEMINI_API_KEY="${GEMINI_API_KEY:-sk-proxy}"
export GEMINI_SANDBOX=false
exec gemini --yolo -p "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
