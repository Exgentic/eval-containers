# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=2.2.8
LABEL eval.type="agent"
LABEL eval.agent.name="mini-swe-agent"
LABEL eval.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/princeton-nlp/SWE-agent"
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
# MSWEA_CONFIGURED skips mini's first-run setup wizard.
# `script -qec` provides a fake-PTY — mini's typer CLI aborts on non-TTY stdin
# even with --yolo.
export MSWEA_CONFIGURED=1
# The task text MUST NOT be interpolated into script's command string: `script -c`
# hands that string to `sh -c`, so a task containing backticks, quotes or newlines
# is re-parsed as shell (observed: DeepSWE instructions carry `require()` and
# 'demo', which produced `sh: 1: Syntax error: end of file unexpected` and exit 2
# before mini ever started). Write the task to a file and have the inner shell read
# it into an argv element via "$(cat ...)" — the command string then contains no
# task bytes at all, only a fixed path.
printf '%s' "${TASK:-}" > /tmp/mswea-task.txt
exec script -qec '/opt/mini-venv/bin/mini --model "openai/${EVAL_MODEL:-default}" --yolo --task "$(cat /tmp/mswea-task.txt)"' /dev/null
E

ENTRYPOINT ["/run.sh"]
