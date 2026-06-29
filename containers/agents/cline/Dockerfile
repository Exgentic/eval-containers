# syntax=docker/dockerfile:1
# Cline agent image (cline.bot)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.15.0
LABEL eval.type="agent"
LABEL eval.agent.name="cline"
LABEL eval.agent.description="Cline autonomous coding agent CLI (Plan/Act modes, MCP integration)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/cline/cline"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=cline
RUN npm install -g cline@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx cline; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# cline's auth state lives in $HOME/.cline/data/{globalState,secrets}.json
# and is only recognised when written via `cline auth`.
cline auth -p openai -k "$OPENAI_API_KEY" -m "${EVAL_MODEL:-default}" -b "$OPENAI_BASE_URL" >/dev/null 2>&1 || true
exec cline -y ${EVAL_AGENT_REASONING_EFFORT:+--reasoning-effort $EVAL_AGENT_REASONING_EFFORT} "$TASK"
E

ENTRYPOINT ["/run.sh"]
