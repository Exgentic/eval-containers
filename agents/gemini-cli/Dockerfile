# syntax=docker/dockerfile:1
# Gemini CLI agent image (Google)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="gemini-cli"
LABEL eval.agent.description="Google Gemini CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/google-gemini/gemini-cli"
LABEL eval.agent.version="0.37.2"
ENV EVAL_AGENT_VERSION_DEFAULT="0.37.2"
ENV EVAL_AGENT=gemini-cli
RUN npm install -g @google/gemini-cli@0.37.2
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx gemini; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN mkdir -p /agent && cat > /agent/run.sh <<'E' && chmod +x /agent/run.sh
#!/bin/bash
set -euo pipefail
export GOOGLE_GEMINI_BASE_URL="${GOOGLE_GEMINI_BASE_URL:-http://model:4000}"
export GEMINI_API_KEY="${GEMINI_API_KEY:-sk-proxy}"
export GEMINI_SANDBOX=false
exec gemini --yolo -p "$TASK"
E
ENTRYPOINT ["/agent/run.sh"]
