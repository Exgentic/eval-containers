# syntax=docker/dockerfile:1
# Codex CLI agent image (OpenAI)
FROM quay.io/eval-containers/core/agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="codex"
LABEL eval.agent.description="OpenAI Codex CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/openai/codex"
LABEL eval.agent.version="0.120.0"
ENV EVAL_AGENT_VERSION_DEFAULT="0.120.0"
ENV EVAL_AGENT=codex
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
model_provider = "eval-containers"
[model_providers.eval-containers]
name = "Eval Containers LiteLLM Proxy"
base_url = "${OPENAI_BASE_URL:-http://model:4000/v1}"
env_key = "OPENAI_API_KEY"
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
