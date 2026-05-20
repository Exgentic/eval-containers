# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)
FROM quay.io/eval-containers/core/agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="openhands"
LABEL eval.agent.description="OpenHands open-source AI software engineer"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/All-Hands-AI/OpenHands"
LABEL eval.agent.version="1.16.0"
ENV EVAL_AGENT_VERSION_DEFAULT="1.16.0"
ENV EVAL_AGENT=openhands
# `openhands` is the CLI package; `openhands-ai` is SDK-only.
# https://docs.openhands.dev/openhands/usage/cli/terminal
RUN uv pip install --system openhands==1.16.0
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
/opt/agent/uv pip install --system openhands==1.16.0
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export LLM_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export LLM_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# litellm needs a provider prefix on LLM_MODEL — add openai/ if absent.
M="${EVAL_MODEL:-default}"; case "$M" in */*) ;; *) M="openai/$M" ;; esac
export LLM_MODEL="$M"
# --override-with-envs makes the CLI use LLM_* env vars instead of the
# interactive Settings panel.
exec openhands --override-with-envs -t "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
