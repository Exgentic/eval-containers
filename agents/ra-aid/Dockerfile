# syntax=docker/dockerfile:1
# RA.Aid agent image
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="ra-aid"
LABEL dock.agent.description="RA.Aid — autonomous research-first coding agent"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/ai-christianson/RA.Aid"
LABEL dock.agent.version="0.30.2"
ENV DOCK_AGENT_VERSION_DEFAULT="0.30.2"
ENV DOCK_AGENT=ra-aid
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update && apt-get install -y --no-install-recommends ripgrep; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/* && \
    pip install --no-cache-dir ra-aid==0.30.2
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y --no-install-recommends ripgrep && rm -rf /var/lib/apt/lists/*
pip install --no-cache-dir ra-aid==0.30.2
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}/v1"
export EXPERT_OPENAI_API_KEY="${OPENAI_API_KEY}"
export EXPERT_OPENAI_API_BASE="${OPENAI_API_BASE}"
exec ra-aid --cowboy-mode --provider openai-compatible --model "${DOCK_MODEL:-default}" \
  --expert-provider openai-compatible --expert-model "${DOCK_MODEL:-default}" -m "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
