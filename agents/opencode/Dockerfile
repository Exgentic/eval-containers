# syntax=docker/dockerfile:1
# opencode agent image (SST)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=1.4.3
LABEL eval.type="agent"
LABEL eval.agent.name="opencode"
LABEL eval.agent.description="SST opencode open-source AI coding agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/sst/opencode"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=opencode
RUN npm install -g opencode-ai@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx opencode; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
mkdir -p "$HOME/.config/opencode"
cat > "$HOME/.config/opencode/opencode.json" <<CONF
{"\$schema":"https://opencode.ai/config.json","model":"eval-containers/${EVAL_MODEL:-default}","small_model":"eval-containers/${EVAL_MODEL:-default}","disabled_providers":["anthropic","openai"],"provider":{"eval-containers":{"npm":"@ai-sdk/openai-compatible","name":"Eval Containers LiteLLM Proxy","options":{"baseURL":"${OPENAI_BASE_URL:-http://model:4000/v1}","apiKey":"${OPENAI_API_KEY:-sk-proxy}"},"models":{"${EVAL_MODEL:-default}":{"name":"Eval Containers Proxy Model"}}}},"permission":{"edit":"allow","bash":"allow","webfetch":"allow"},"autoshare":false}
CONF
# Benchmark-declared MCP servers (doctrine/agents rule 19). opencode reads
# them from the same opencode.json written above, so we merge in rather
# than rewrite — the provider block is what lets the agent reach the
# gateway at all. `"type":"remote"` is opencode's name for streamable
# HTTP; `oauth:false` keeps it from attempting a browser auth flow
# against a server that never asked for one. `permission["*"]` is needed
# because `opencode run` auto-*rejects* any tool ask it can't prompt for,
# and the base config only allows edit/bash/webfetch by name.
# Unset or `{}` leaves the file exactly as the heredoc wrote it.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  node -e 'const fs = require("fs"), p = process.env.HOME + "/.config/opencode/opencode.json"; const c = JSON.parse(fs.readFileSync(p, "utf8")); c.mcp = Object.fromEntries(Object.entries(JSON.parse(process.env.EVAL_MCP_SERVERS)).map(([n, u]) => [n, {type: "remote", url: u, enabled: true, oauth: false}])); c.permission = {...c.permission, "*": "allow"}; fs.writeFileSync(p, JSON.stringify(c))'
fi
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1
exec opencode run --model "eval-containers/${EVAL_MODEL:-default}" "$TASK"
E

ENTRYPOINT ["/run.sh"]
