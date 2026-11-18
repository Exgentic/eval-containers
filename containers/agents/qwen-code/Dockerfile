# syntax=docker/dockerfile:1
# Qwen Code agent image (Alibaba)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.14.4
LABEL eval.type="agent"
LABEL eval.agent.name="qwen-code"
LABEL eval.agent.description="Qwen Code — Alibaba's terminal AI coding agent (gemini-cli fork)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/QwenLM/qwen-code"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=qwen-code
RUN npm install -g @qwen-code/qwen-code@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx qwen; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
mkdir -p "$HOME/.qwen"
DM="${MODEL:-default}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000/v1}"
export OPENAI_MODEL="$DM"
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) is the model's contextWindowSize
# (131072 unless told), and qwen keeps its own compression: before each request,
# once the last prompt passes 70% of it. A reply starts capped at 8000 and is
# retried once at 64000 if cut there, so 64000 must fit in the other 30%. When it
# doesn't, QWEN_CODE_MAX_OUTPUT_TOKENS becomes window x 3/10, which also turns
# that retry off; a lower cap the launch set is kept.
GC=""
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  W=$EVAL_MODEL_CONTEXT_WINDOW
  GC=",\"generationConfig\":{\"contextWindowSize\":$W}"
  CAP=${QWEN_CODE_MAX_OUTPUT_TOKENS:-64000}
  [[ "$CAP" =~ ^[1-9][0-9]*$ ]] || CAP=64000
  [ "$CAP" -le $(( W * 3 / 10 )) ] || export QWEN_CODE_MAX_OUTPUT_TOKENS=$(( W * 3 / 10 ))
fi
cat > "$HOME/.qwen/settings.json" <<CONF
{"modelProviders":{"openai":[{"id":"$DM","name":"$DM","baseUrl":"$OPENAI_BASE_URL","envKey":"OPENAI_API_KEY"$GC}]},"security":{"auth":{"selectedType":"openai"}},"model":{"name":"$DM"}}
CONF
export GEMINI_SANDBOX=false
export QWEN_SANDBOX=false
exec qwen --yolo -p "$TASK"
E

ENTRYPOINT ["/run.sh"]
