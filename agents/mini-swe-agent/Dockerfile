# syntax=docker/dockerfile:1
# mini-swe-agent — lightweight AI coding agent
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=2.2.8
LABEL eval.type="agent"
LABEL eval.agent.name="mini-swe-agent"
LABEL eval.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/princeton-nlp/SWE-agent"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19) — mini's only tool
# interface is a shell, so the servers are reached through the
# `mcp-bridge` command below. The label is for discovery only;
# /opt/agent/MCP is what the runtime guard reads, because labels do not
# survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=mini-swe-agent
RUN uv pip install --system --no-cache mini-swe-agent==${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
COPY mcp-bridge /opt/agent/mcp-bridge
RUN chmod +x /opt/agent/mcp-bridge
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir mini-swe-agent==${AGENT_VERSION}
# The agent reaches MCP servers by running `mcp` in its shell, so the
# bridge has to be on PATH for the agent's own commands, not just ours.
ln -sf /opt/agent/mcp-bridge /usr/local/bin/mcp
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
# Benchmark-declared MCP servers (doctrine/agents rule 19). mini has no
# MCP client and no tool registry — the model acts by writing shell
# commands — so the servers arrive as a documented command plus an
# inventory appended to the task. Listing the tools here, before the
# model runs, is deliberate: it is the only way the agent learns what
# exists, and it means the `tools/list` handshake does not depend on the
# model choosing to look. Unset or `{}` appends nothing.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  MCP_TOOLS="$(/opt/agent/mcp-bridge prompt || true)"
  if [ -n "$MCP_TOOLS" ]; then
    TASK="$TASK
$MCP_TOOLS"
  fi
fi
exec script -qec "mini --model openai/${EVAL_MODEL:-default} --yolo --task \"$TASK\"" /dev/null
E

ENTRYPOINT ["/run.sh"]
