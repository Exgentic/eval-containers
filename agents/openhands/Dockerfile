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
# All-Hands-AI's package is `openhands-ai` (not plain `openhands`) — the
# `openhands` namespace on PyPI is owned by a different sign-language
# project (AI4Bharat), so `pip install openhands==X` resolves to a stub
# (`from versions: 0.0.0`). Pin to 1.7.0, latest on openhands-ai as of
# audit time (May 2026).
#
# `lmnr` (Laminar tracing, transitive dep of openhands-ai) pins
# opentelemetry-semantic-conventions==0.60b1 at version 0.7.47, but
# 0.7.51 (current) accepts a broader range and resolves against the
# opentelemetry-instrumentation 0.6xb releases. Pin lmnr to 0.7.51
# explicitly so pip's resolver doesn't get stuck backtracking through
# every 0.7.4x release.
RUN pip install --no-cache-dir openhands-ai==1.7.0 lmnr==0.7.51
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir openhands-ai==1.7.0 lmnr==0.7.51
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
