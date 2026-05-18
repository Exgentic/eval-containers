# syntax=docker/dockerfile:1
# opencode agent image (SST)
FROM quay.io/eval-containers/core/agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="opencode"
LABEL eval.agent.description="SST opencode open-source AI coding agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/sst/opencode"
LABEL eval.agent.version="1.4.3"
ENV EVAL_AGENT_VERSION_DEFAULT="1.4.3"
ENV EVAL_AGENT=opencode
RUN npm install -g opencode-ai@1.4.3
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx opencode; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
mkdir -p "$HOME/.config/opencode"
cat > "$HOME/.config/opencode/opencode.json" <<CONF
{"\$schema":"https://opencode.ai/config.json","model":"eval-containers/${EVAL_MODEL:-default}","small_model":"eval-containers/${EVAL_MODEL:-default}","disabled_providers":["anthropic","openai"],"provider":{"eval-containers":{"npm":"@ai-sdk/openai-compatible","name":"Eval Containers LiteLLM Proxy","options":{"baseURL":"${OPENAI_BASE_URL:-http://model:4000}/v1","apiKey":"${OPENAI_API_KEY:-sk-proxy}"},"models":{"${EVAL_MODEL:-default}":{"name":"Eval Containers Proxy Model"}}}},"permission":{"edit":"allow","bash":"allow","webfetch":"allow"},"autoshare":false}
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1
exec opencode run --model "eval-containers/${EVAL_MODEL:-default}" "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
