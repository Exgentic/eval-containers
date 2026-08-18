# syntax=docker/dockerfile:1
# GitHub Copilot CLI agent image
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=1.0.24
LABEL eval.type="agent"
LABEL eval.agent.name="copilot-cli"
LABEL eval.agent.description="GitHub Copilot CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/github/copilot-cli"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=copilot-cli
RUN npm install -g @github/copilot@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx copilot; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export COPILOT_PROVIDER_BASE_URL="${OPENAI_BASE_URL:-http://model:4000/v1}"
export COPILOT_PROVIDER_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export COPILOT_OFFLINE=true
export COPILOT_MODEL="${COPILOT_MODEL:-default}"
# Benchmark-declared MCP servers (doctrine/agents rule 19). copilot takes
# them as a session-scoped JSON blob, which beats writing a config file:
# it has the highest precedence and cannot be shadowed by a stray
# ~/.copilot/mcp-config.json in some benchmark base. `type` is mandatory
# at this pin, and `tools: ["*"]` pre-allows the server's tools so the
# run doesn't stall on a consent prompt nothing can answer.
# Unset or `{}` adds no flags, leaving the invocation byte-identical.
MCP=()
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  MCP=(--additional-mcp-config "$(node -e 'const m = JSON.parse(process.env.EVAL_MCP_SERVERS); console.log(JSON.stringify({mcpServers: Object.fromEntries(Object.entries(m).map(([n, u]) => [n, {type: "http", url: u, tools: ["*"]}]))}))')")
fi
echo "$TASK" | exec copilot --yolo "${MCP[@]}"
E

ENTRYPOINT ["/run.sh"]
