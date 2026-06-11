# syntax=docker/dockerfile:1
# IBM Bob Shell agent image
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=1.0.1
LABEL eval.type="agent"
LABEL eval.agent.name="bob"
LABEL eval.agent.description="IBM Bob Shell — AI SDLC partner"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/ibm/bob-shell"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=bob
ARG BOB_VERSION=${AGENT_VERSION}
RUN curl -fsSL "https://s3.us-south.cloud-object-storage.appdomain.cloud/bob-shell/bobshell-${BOB_VERSION}.tgz" -o /tmp/bob.tgz && \
    npm install -g /tmp/bob.tgz && rm /tmp/bob.tgz
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx bob; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export BOB_ACCEPT_LICENSE=1
export BOB_TELEMETRY=off
exec bob --yolo --accept-license "$TASK"
E

ENTRYPOINT ["/run.sh"]
