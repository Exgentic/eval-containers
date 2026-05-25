# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)
ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="terminus-2"
LABEL eval.agent.description="Harbor Terminus-2 reference agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/harbor-ai/harbor"
LABEL eval.agent.version="0.6.4"
ENV EVAL_AGENT_VERSION_DEFAULT="0.6.4"
ENV EVAL_AGENT=terminus-2
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update && apt-get install -y --no-install-recommends tmux; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/* && \
    pip install --no-cache-dir harbor==0.6.4
COPY run.py /opt/agent/run.py
RUN chmod +x /opt/agent/run.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y --no-install-recommends tmux && rm -rf /var/lib/apt/lists/*
pip install --no-cache-dir harbor==0.6.4
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
exec python3 /opt/agent/run.py
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
