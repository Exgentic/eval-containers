# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)
FROM quay.io/eval-containers/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="openhands"
LABEL eval.agent.description="OpenHands open-source AI software engineer"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL eval.agent.version="1.7.0"
ENV EVAL_AGENT_VERSION_DEFAULT="1.7.0"
ENV EVAL_AGENT=openhands
# openhands-ai requires Python >=3.12; several benchmark bases ship 3.10
# (aider-polyglot's ubuntu:22.04). Provision a self-contained venv via uv's
# managed Python and run from it — works regardless of the base's system
# Python. Single install.sh is the source of truth for both the agent image
# and combo builds. We use the SDK package (openhands-ai) wrapped by
# run_openhands.py as a one-shot CLI.
COPY run_openhands.py /opt/agent/run_openhands.py
RUN chmod +x /opt/agent/run_openhands.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
# UV_PYTHON_INSTALL_DIR moves managed Python out of /root (mode 0700 in the
# combo image, see core/combination.Dockerfile rationale) so uid 1002 can
# traverse the venv's symlink chain back to the interpreter at runtime.
export UV_PYTHON_INSTALL_DIR=/opt/uv-python
command -v uv >/dev/null 2>&1 || pip install --no-cache-dir --quiet uv
uv python install --quiet 3.12
uv venv --quiet --python 3.12 /opt/openhands-venv
uv pip install --quiet --python /opt/openhands-venv/bin/python openhands-ai==1.7.0 lmnr==0.7.51
chmod -R a+rX /opt/uv-python /opt/openhands-venv
I
RUN /opt/agent/install.sh
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export LLM_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export LLM_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export LLM_MODEL="${EVAL_MODEL:-default}"
exec /opt/openhands-venv/bin/python /opt/agent/run_openhands.py
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
