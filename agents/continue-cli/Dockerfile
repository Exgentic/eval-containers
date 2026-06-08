# syntax=docker/dockerfile:1
# Continue CLI agent image (continuedev/continue)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=1.5.45
LABEL eval.type="agent"
LABEL eval.agent.name="continue-cli"
LABEL eval.agent.description="Continue CLI (cn): terminal-native multi-model coding agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/continuedev/continue"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=continue-cli
RUN npm install -g @continuedev/cli@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx cn; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
D="${HOME:-/root}/.continue"
mkdir -p "$D"
cat > "$D/config.yaml" <<CONF
name: eval-proxy
version: 0.0.1
schema: v1
models:
  - name: default
    provider: openai
    model: ${EVAL_MODEL:-default}
    apiKey: ${OPENAI_API_KEY}
    apiBase: ${OPENAI_BASE_URL}
    roles: [chat, edit, apply]
CONF
exec cn -p "$TASK"
E

ENTRYPOINT ["/run.sh"]
