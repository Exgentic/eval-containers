# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="openhands"
LABEL dock.agent.description="OpenHands open-source AI software engineer"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL dock.agent.version="1.14.0"
ENV DOCK_AGENT_VERSION_DEFAULT="1.14.0"
ENV DOCK_AGENT=openhands
RUN pip install --no-cache-dir openhands==1.14.0
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir openhands==1.14.0
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export LLM_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export LLM_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export LLM_MODEL="${DOCK_MODEL:-gpt-4o}"
exec openhands --headless --always-approve --override-with-envs -t "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
