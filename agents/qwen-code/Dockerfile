# syntax=docker/dockerfile:1
# Qwen Code agent image (Alibaba)
FROM quay.io/eval-containers/core/agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="qwen-code"
LABEL eval.agent.description="Qwen Code — Alibaba's terminal AI coding agent (gemini-cli fork)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/QwenLM/qwen-code"
LABEL eval.agent.version="0.14.4"
ENV EVAL_AGENT_VERSION_DEFAULT="0.14.4"
ENV EVAL_AGENT=qwen-code
RUN npm install -g @qwen-code/qwen-code@0.14.4
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx qwen; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
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
export GEMINI_SANDBOX=false
export QWEN_SANDBOX=false
exec qwen --yolo -p "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
