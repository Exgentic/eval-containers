# syntax=docker/dockerfile:1
# Continue CLI agent image (continuedev/continue)
FROM quay.io/dock-eval/core/agent-base-node:latest
LABEL dock.type="agent"
LABEL dock.agent.name="continue-cli"
LABEL dock.agent.description="Continue CLI (cn): terminal-native multi-model coding agent"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/continuedev/continue"
LABEL dock.agent.version="1.5.45"
ENV DOCK_AGENT_VERSION_DEFAULT="1.5.45"
ENV DOCK_AGENT=continue-cli
RUN npm install -g @continuedev/cli@1.5.45
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx cn; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
D="${HOME:-/root}/.continue"
mkdir -p "$D"
cat > "$D/config.yaml" <<CONF
name: dock-proxy
version: 0.0.1
schema: v1
models:
  - name: default
    provider: openai
    model: ${DOCK_MODEL:-default}
    apiKey: ${OPENAI_API_KEY}
    apiBase: ${OPENAI_BASE_URL}
    roles: [chat, edit, apply]
CONF
exec cn -p "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
