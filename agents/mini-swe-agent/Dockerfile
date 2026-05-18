# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
FROM quay.io/eval-containers/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="mini-swe-agent"
LABEL eval.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/princeton-nlp/SWE-agent"
LABEL eval.agent.version="2.2.8"
ENV EVAL_AGENT_VERSION_DEFAULT="2.2.8"
ENV EVAL_AGENT=mini-swe-agent
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
exec mini --model "openai/${EVAL_MODEL:-default}" --yolo --task "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
