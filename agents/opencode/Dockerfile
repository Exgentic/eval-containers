# syntax=docker/dockerfile:1
# opencode agent image (SST)
FROM quay.io/dock-eval/core/agent-base-node:latest
LABEL dock.type="agent"
LABEL dock.agent.name="opencode"
LABEL dock.agent.description="SST opencode open-source AI coding agent"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/sst/opencode"
LABEL dock.agent.version="1.4.3"
ENV DOCK_AGENT_VERSION_DEFAULT="1.4.3"
ENV DOCK_AGENT=opencode
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
{"\$schema":"https://opencode.ai/config.json","model":"dock/${DOCK_MODEL:-default}","small_model":"dock/${DOCK_MODEL:-default}","disabled_providers":["anthropic","openai"],"provider":{"dock":{"npm":"@ai-sdk/openai-compatible","name":"Dock LiteLLM Proxy","options":{"baseURL":"${OPENAI_BASE_URL:-http://model:4000}/v1","apiKey":"${OPENAI_API_KEY:-sk-proxy}"},"models":{"${DOCK_MODEL:-default}":{"name":"Dock Proxy Model"}}}},"permission":{"edit":"allow","bash":"allow","webfetch":"allow"},"autoshare":false}
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1
exec opencode run --model "dock/${DOCK_MODEL:-default}" "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
