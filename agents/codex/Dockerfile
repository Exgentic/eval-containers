# syntax=docker/dockerfile:1
# Codex CLI agent image (OpenAI)
FROM quay.io/dock-eval/core/agent-base-node:latest
LABEL dock.type="agent"
LABEL dock.agent.name="codex"
LABEL dock.agent.description="OpenAI Codex CLI"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/openai/codex"
LABEL dock.agent.version="0.120.0"
ENV DOCK_AGENT_VERSION_DEFAULT="0.120.0"
ENV DOCK_AGENT=codex
RUN npm install -g @openai/codex@0.120.0
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx codex; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
mkdir -p ~/.codex
cat > ~/.codex/config.toml <<CONF
model = "${CODEX_MODEL:-default}"
model_provider = "dock"
[model_providers.dock]
name = "Dock LiteLLM Proxy"
base_url = "${OPENAI_BASE_URL:-http://model:4000}/v1"
env_key = "OPENAI_API_KEY"
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
