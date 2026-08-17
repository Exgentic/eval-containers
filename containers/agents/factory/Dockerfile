# syntax=docker/dockerfile:1
# Factory Droid CLI agent image
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.197.0
LABEL eval.type="agent"
LABEL eval.agent.name="factory"
LABEL eval.agent.description="Factory Droid CLI — autonomous software engineering agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://docs.factory.ai/droid-cli/getting-started/overview"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=factory
# npm installs disable droid's self-update, so the pin holds at runtime (rule 12).
ENV FACTORY_DROID_AUTO_UPDATE_ENABLED=false
RUN npm install -g droid@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
for b in node npm npx droid; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# droid ignores OPENAI_BASE_URL; the gateway is reachable only via a BYOK
# customModels entry in settings.json, keyed by the model id we pass to -m.
BASE="${OPENAI_BASE_URL:-http://model:4000/v1}"
MODEL="${EVAL_MODEL:-default}"
mkdir -p "$HOME/.factory"
cat > "$HOME/.factory/settings.json" <<J
{"customModels":[{"model":"${MODEL}","displayName":"Eval Containers Gateway","baseUrl":"${BASE}","apiKey":"${OPENAI_API_KEY}","provider":"generic-chat-completion-api","maxOutputTokens":16384}]}
J
# --skip-permissions-unsafe: Docker is the sandbox (rule 10).
exec droid exec \
  --skip-permissions-unsafe \
  -m "$MODEL" \
  -o text \
  ${EVAL_AGENT_REASONING_EFFORT:+-r $EVAL_AGENT_REASONING_EFFORT} \
  "$TASK"
E

ENTRYPOINT ["/run.sh"]
