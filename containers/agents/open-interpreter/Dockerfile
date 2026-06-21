# syntax=docker/dockerfile:1
# Open Interpreter agent image
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.4.3
LABEL eval.type="agent"
LABEL eval.agent.name="open-interpreter"
LABEL eval.agent.description="Open Interpreter: natural-language code execution agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/OpenInterpreter/open-interpreter"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=open-interpreter
RUN uv pip install --system --no-cache 'setuptools<81' open-interpreter==${AGENT_VERSION}
COPY run_oi.py /opt/agent/run_oi.py
RUN chmod +x /opt/agent/run_oi.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
/opt/agent/setup-uv-venv.sh /opt/oi-venv 'setuptools<81' open-interpreter==${AGENT_VERSION}
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL}"
exec /opt/oi-venv/bin/python /opt/agent/run_oi.py "$TASK"
E

ENTRYPOINT ["/run.sh"]
