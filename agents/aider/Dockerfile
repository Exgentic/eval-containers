# syntax=docker/dockerfile:1
# Aider agent image (AI pair programming tool)
FROM quay.io/eval-containers/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="aider"
LABEL eval.agent.description="Aider AI pair programming tool"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/Aider-AI/aider"
LABEL eval.agent.version="0.86.2"
ENV EVAL_AGENT_VERSION_DEFAULT="0.86.2"
ENV EVAL_AGENT=aider
RUN uv pip install --system aider-chat==0.86.2
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
/opt/agent/uv pip install --system aider-chat==0.86.2
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
exec aider --yes-always --no-git --no-auto-commits --no-stream \
  --openai-api-base "${OPENAI_BASE_URL:-http://model:4000}" \
  --model "openai/${EVAL_MODEL:-default}" --message "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
