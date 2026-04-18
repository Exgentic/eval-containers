# syntax=docker/dockerfile:1
# Aider agent image (AI pair programming tool)
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="aider"
LABEL dock.agent.description="Aider AI pair programming tool"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/Aider-AI/aider"
LABEL dock.agent.version="0.86.2"
ENV DOCK_AGENT_VERSION_DEFAULT="0.86.2"
ENV DOCK_AGENT=aider
RUN pip install --no-cache-dir aider-chat==0.86.2
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir aider-chat==0.86.2
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec aider --yes-always --no-git --no-auto-commits --no-stream \
  --openai-api-base "${OPENAI_BASE_URL:-http://model:4000}" \
  --model "openai/${DOCK_MODEL:-default}" --message "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
