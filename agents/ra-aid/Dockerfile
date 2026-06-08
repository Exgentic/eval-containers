# syntax=docker/dockerfile:1
# RA.Aid agent image
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.19.1
LABEL eval.type="agent"
LABEL eval.agent.name="ra-aid"
LABEL eval.agent.description="RA.Aid — autonomous research-first coding agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/ai-christianson/RA.Aid"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=ra-aid
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update && apt-get install -y --no-install-recommends ripgrep; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/* && \
    uv pip install --system --no-cache 'langgraph<0.4' ra-aid==${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y --no-install-recommends ripgrep && rm -rf /var/lib/apt/lists/*
pip install --no-cache-dir 'langgraph<0.4' ra-aid==${AGENT_VERSION}
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000/v1}"
export EXPERT_OPENAI_API_KEY="${OPENAI_API_KEY}"
export EXPERT_OPENAI_API_BASE="${OPENAI_API_BASE}"
exec ra-aid --cowboy-mode --provider openai-compatible --model "${EVAL_MODEL:-default}" \
  --expert-provider openai-compatible --expert-model "${EVAL_MODEL:-default}" -m "$TASK"
E

ENTRYPOINT ["/run.sh"]
