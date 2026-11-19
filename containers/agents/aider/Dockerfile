# syntax=docker/dockerfile:1
# Aider agent image (AI pair programming tool)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.86.2
LABEL eval.type="agent"
LABEL eval.agent.name="aider"
LABEL eval.agent.description="Aider AI pair programming tool"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/Aider-AI/aider"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=aider
RUN uv pip install --system --no-cache aider-chat==${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
/opt/agent/setup-uv-venv.sh /opt/aider-venv aider-chat==${AGENT_VERSION}
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) is the model's max_input_tokens, in
# a model metadata file (aider knows no window for a model it doesn't know). aider
# keeps its own summarizing: chat history past window/16 (1024 to 8192 tokens) is
# summarized, and a prompt over the window is reported. It sends no reply cap, so
# nothing is reserved.
META=""
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  META=$(mktemp)
  printf '{"openai/%s":{"max_input_tokens":%d}}\n' "${MODEL:-default}" "$EVAL_MODEL_CONTEXT_WINDOW" > "$META"
fi
exec /opt/aider-venv/bin/aider --yes-always --no-git --no-auto-commits --no-stream \
  --openai-api-base "${OPENAI_BASE_URL:-http://model:4000}" ${META:+--model-metadata-file "$META"} \
  --model "openai/${MODEL:-default}" ${EVAL_AGENT_REASONING_EFFORT:+--reasoning-effort $EVAL_AGENT_REASONING_EFFORT} --message "$TASK"
E

ENTRYPOINT ["/run.sh"]
