# syntax=docker/dockerfile:1
# GitHub Copilot CLI agent image
FROM quay.io/eval-containers/core/agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="copilot-cli"
LABEL eval.agent.description="GitHub Copilot CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/github/copilot-cli"
LABEL eval.agent.version="1.0.24"
ENV EVAL_AGENT_VERSION_DEFAULT="1.0.24"
ENV EVAL_AGENT=copilot-cli
RUN npm install -g @github/copilot@1.0.24
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx copilot; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export COPILOT_PROVIDER_BASE_URL="${OPENAI_BASE_URL:-http://model:4000/v1}"
export COPILOT_PROVIDER_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export COPILOT_OFFLINE=true
export COPILOT_MODEL="${COPILOT_MODEL:-default}"
echo "$TASK" | exec copilot --yolo
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
