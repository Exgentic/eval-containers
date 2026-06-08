# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=1.7.0
LABEL eval.type="agent"
LABEL eval.agent.name="openhands"
LABEL eval.agent.description="OpenHands open-source AI software engineer"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=openhands
# openhands-ai requires Python >=3.12; several benchmark bases ship 3.10
# (aider-polyglot's ubuntu:22.04). Provision a self-contained venv via the
# shared `/opt/agent/setup-uv-venv.sh` helper (core/agent-base-python). We
# use the SDK package (openhands-ai) wrapped by run_openhands.py as a
# one-shot CLI.
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
/opt/agent/setup-uv-venv.sh /opt/openhands-venv openhands-ai==${AGENT_VERSION} lmnr==0.7.51
I

RUN /opt/agent/install.sh
# Late COPY: editing run_openhands.py here invalidates only the layers
# below, not the multi-minute venv install above.
COPY run_openhands.py /opt/agent/run_openhands.py
RUN chmod +x /opt/agent/run_openhands.py
RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export LLM_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export LLM_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export LLM_MODEL="${EVAL_MODEL:-default}"
exec /opt/openhands-venv/bin/python /opt/agent/run_openhands.py
E

ENTRYPOINT ["/run.sh"]
