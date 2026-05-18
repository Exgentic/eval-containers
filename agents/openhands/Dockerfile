# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)
FROM quay.io/eval-containers/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="openhands"
LABEL eval.agent.description="OpenHands open-source AI software engineer"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL eval.agent.version="1.14.0"
ENV EVAL_AGENT_VERSION_DEFAULT="1.14.0"
ENV EVAL_AGENT=openhands
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
export LLM_MODEL="${EVAL_MODEL:-gpt-4o}"
exec openhands --headless --always-approve --override-with-envs -t "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
