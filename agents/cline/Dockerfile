# syntax=docker/dockerfile:1
# Cline agent image (cline.bot)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="cline"
LABEL eval.agent.description="Cline autonomous coding agent CLI (Plan/Act modes, MCP integration)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/cline/cline"
LABEL eval.agent.version="2.15.0"
ENV EVAL_AGENT_VERSION_DEFAULT="2.15.0"
ENV EVAL_AGENT=cline
RUN npm install -g cline@2.15.0
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx cline; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# cline's auth state lives in $HOME/.cline/data/{globalState,secrets}.json
# and is only recognised when written via `cline auth`.
cline auth -p openai -k "$OPENAI_API_KEY" -m "${EVAL_MODEL:-default}" -b "$OPENAI_BASE_URL" >/dev/null 2>&1 || true
exec cline -y "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
