# syntax=docker/dockerfile:1
# Qwen Code agent image (Alibaba)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.14.4
LABEL eval.type="agent"
LABEL eval.agent.name="qwen-code"
LABEL eval.agent.description="Qwen Code — Alibaba's terminal AI coding agent (gemini-cli fork)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/QwenLM/qwen-code"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=qwen-code
RUN npm install -g @qwen-code/qwen-code@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx qwen; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
mkdir -p "$HOME/.qwen"
DM="${EVAL_MODEL:-default}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000/v1}"
export OPENAI_MODEL="$DM"
cat > "$HOME/.qwen/settings.json" <<CONF
{"modelProviders":{"openai":[{"id":"$DM","name":"$DM","baseUrl":"$OPENAI_BASE_URL","envKey":"OPENAI_API_KEY"}]},"security":{"auth":{"selectedType":"openai"}},"model":{"name":"$DM"}}
CONF
# Benchmark-declared MCP servers (doctrine/agents rule 19). qwen-code
# reads `mcpServers` from the same settings.json written above, so we
# merge into it rather than rewriting — the provider/auth block is what
# makes the agent able to reach the gateway at all. This fork still uses
# gemini-cli's older `httpUrl` key for streamable HTTP; `trust` skips the
# per-tool confirmation `--yolo` does not cover for MCP. Unset or `{}`
# leaves the file exactly as the heredoc wrote it.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  node -e 'const fs = require("fs"), p = process.env.HOME + "/.qwen/settings.json"; const c = JSON.parse(fs.readFileSync(p, "utf8")); c.mcpServers = Object.fromEntries(Object.entries(JSON.parse(process.env.EVAL_MCP_SERVERS)).map(([n, u]) => [n, {httpUrl: u, trust: true}])); fs.writeFileSync(p, JSON.stringify(c))'
fi
export GEMINI_SANDBOX=false
export QWEN_SANDBOX=false
exec qwen --yolo -p "$TASK"
E

ENTRYPOINT ["/run.sh"]
