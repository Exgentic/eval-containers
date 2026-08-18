# syntax=docker/dockerfile:1
# RA.Aid agent image
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.19.1
LABEL eval.type="agent"
LABEL eval.agent.name="ra-aid"
LABEL eval.agent.description="RA.Aid — autonomous research-first coding agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/ai-christianson/RA.Aid"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19) — rendered as a
# `--custom-tools` module (mcp_tools.py) since RA.Aid's own MCP client is
# broken at the versions we pin. The label is for discovery only;
# /opt/agent/MCP is what the runtime guard reads, because labels do not
# survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=ra-aid
RUN echo true > /opt/agent/MCP
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update && apt-get install -y --no-install-recommends ripgrep; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/* && \
    uv pip install --system --no-cache 'langgraph<0.4' ra-aid==${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y --no-install-recommends ripgrep && rm -rf /var/lib/apt/lists/*
pip install --no-cache-dir 'langgraph<0.4' ra-aid==${AGENT_VERSION}
# The MCP tool wrapper also runs as a shell command (`mcp call ...`) when
# a tool needs to be exercised outside LangChain, so put it on PATH.
ln -sf /opt/agent/mcp-bridge /usr/local/bin/mcp
I

# Late COPY: editing these invalidates only the layers below, not the
# multi-minute apt + pip install above.
COPY mcp-bridge /opt/agent/mcp-bridge
COPY mcp_tools.py /opt/agent/mcp_tools.py
RUN chmod +x /opt/agent/mcp-bridge

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000/v1}"
export EXPERT_OPENAI_API_KEY="${OPENAI_API_KEY}"
export EXPERT_OPENAI_API_BASE="${OPENAI_API_BASE}"
# Benchmark-declared MCP servers (doctrine/agents rule 19): mcp_tools.py
# renders every entry of the map into a LangChain tool at import time, and
# `--custom-tools` is how RA.Aid takes them. Unset or `{}` adds no flag,
# leaving the invocation byte-identical to the no-MCP case.
MCP=()
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  MCP=(--custom-tools /opt/agent/mcp_tools.py)
fi
exec ra-aid --cowboy-mode --provider openai-compatible --model "${EVAL_MODEL:-default}" \
  --expert-provider openai-compatible --expert-model "${EVAL_MODEL:-default}" \
  "${MCP[@]}" -m "$TASK"
E

ENTRYPOINT ["/run.sh"]
