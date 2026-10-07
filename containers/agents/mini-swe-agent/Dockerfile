# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=2.4.6
LABEL eval.type="agent"
LABEL eval.agent.name="mini-swe-agent"
LABEL eval.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/SWE-agent/mini-swe-agent"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=mini-swe-agent
RUN uv pip install --system --no-cache mini-swe-agent==${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
/opt/agent/setup-uv-venv.sh /opt/mini-venv mini-swe-agent==${AGENT_VERSION}
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
# Model name comes from $MODEL: run-agent scrubs the env (`env -i`) and maps
# EVAL_MODEL → MODEL, so the agent never sees EVAL_MODEL itself.
# MSWEA_CONFIGURED skips mini's first-run setup wizard.
# MSWEA_COST_TRACKING=ignore_errors + --cost-limit 0: mini's litellm cost lookup
# raises for any model name absent from litellm's cost map, and its default $3
# cap would stop the run on mini's own estimate. The gateway is the cost
# authority (EVAL_MODEL_MAX_BUDGET) and run-agent enforces the timeout.
# MSWEA_GLOBAL_CONFIG_DIR keeps mini's config + trajectory under /tmp (rule 8).
# --exit-immediately: without it mini prompts for confirmation after the agent
# submits, and nobody is there to answer.
export MSWEA_CONFIGURED=1
export MSWEA_COST_TRACKING=ignore_errors
export MSWEA_GLOBAL_CONFIG_DIR=/tmp/mini-swe-agent
exec /opt/mini-venv/bin/mini --model "openai/${MODEL:-default}" --yolo --exit-immediately \
  --cost-limit 0 --task "$TASK"
E

ENTRYPOINT ["/run.sh"]
