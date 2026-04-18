# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="mini-swe-agent"
LABEL dock.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/princeton-nlp/SWE-agent"
LABEL dock.agent.version="2.2.8"
ENV DOCK_AGENT_VERSION_DEFAULT="2.2.8"
ENV DOCK_AGENT=mini-swe-agent
RUN pip install --no-cache-dir mini-swe-agent==2.2.8
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir mini-swe-agent==2.2.8
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
exec mini --model "openai/${DOCK_MODEL:-default}" --yolo --task "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
