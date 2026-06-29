# syntax=docker/dockerfile:1
# Codex CLI agent image (OpenAI)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.120.0
LABEL eval.type="agent"
LABEL eval.agent.name="codex"
LABEL eval.agent.description="OpenAI Codex CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/openai/codex"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=codex
RUN npm install -g @openai/codex@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx codex; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
# web_search must be the top-level key (codex ignores [tools].web_search).
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec codex exec \
  --model "${CODEX_MODEL:-default}" \
  -c model_provider=eval-containers \
  -c 'model_providers.eval-containers.name="Eval Containers LiteLLM Proxy"' \
  -c "model_providers.eval-containers.base_url=\"${OPENAI_BASE_URL:-http://model:4000/v1}\"" \
  -c 'model_providers.eval-containers.env_key="OPENAI_API_KEY"' \
  -c 'web_search="live"' \
  -c 'tools.view_image=true' \
  ${EVAL_AGENT_REASONING_EFFORT:+-c model_reasoning_effort=$EVAL_AGENT_REASONING_EFFORT} \
  --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check "$TASK"
E

ENTRYPOINT ["/run.sh"]
