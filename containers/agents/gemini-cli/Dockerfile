# syntax=docker/dockerfile:1
# Gemini CLI agent image (Google)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.37.2
LABEL eval.type="agent"
LABEL eval.agent.name="gemini-cli"
LABEL eval.agent.description="Google Gemini CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/google-gemini/gemini-cli"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=gemini-cli
RUN npm install -g @google/gemini-cli@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx gemini; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export GOOGLE_GEMINI_BASE_URL="${GOOGLE_GEMINI_BASE_URL:-http://model:4000}"
export GEMINI_API_KEY="${GEMINI_API_KEY:-sk-proxy}"
export GEMINI_SANDBOX=false
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26). gemini-cli 0.37.2 has no window
# setting: it takes every model's window as 1048576 and compresses, before each
# request, once the last prompt passes model.compressionThreshold (0.5) of it. So
# the threshold becomes 0.5 x window / 1048576, which keeps gemini's own point,
# half the real window. It sends no reply cap, so nothing is reserved.
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  T=$(( EVAL_MODEL_CONTEXT_WINDOW * 1000000 / 2097152 ))
  mkdir -p "$HOME/.gemini"
  printf '{"model":{"compressionThreshold":%d.%06d}}\n' $(( T / 1000000 )) $(( T % 1000000 )) > "$HOME/.gemini/settings.json"
fi
exec gemini --yolo -p "$TASK"
E

ENTRYPOINT ["/run.sh"]
