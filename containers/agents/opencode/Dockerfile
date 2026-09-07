# syntax=docker/dockerfile:1
# opencode agent image (SST)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=1.4.3
LABEL eval.type="agent"
LABEL eval.agent.name="opencode"
LABEL eval.agent.description="SST opencode open-source AI coding agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/sst/opencode"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=opencode
RUN npm install -g opencode-ai@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
for b in node npm npx opencode; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
mkdir -p "$HOME/.config/opencode"

# Native MCP wiring (parity with the openhands runner, and the eval-containers
# benchmark-agnostic contract — rule 2/4): a benchmark that exposes tools over
# MCP advertises the endpoint as prose inside $TASK, in a `=== MCP SERVER ===`
# block. Parse the url out of that block and wire it as a native opencode
# `remote` MCP server so the agent calls the 82 tools directly instead of
# bootstrapping an MCP client over bash. No block (a non-MCP benchmark, or the
# agents-smoke carrier) -> empty fragment -> behavior is unchanged.
# Match the first whitespace-delimited token after `url:` inside the block —
# equivalent to run_openhands.py's `re.search(r"url:\s*(\S+)")`, so both agents
# extract the identical endpoint from the same prose.
MCP_URL="$(printf '%s\n' "$TASK" | awk '/=== MCP SERVER ===/{f=1} f && /url:/ {sub(/^.*url:[[:space:]]*/,""); print $1; exit}')"
MCP_JSON=""
if [ -n "$MCP_URL" ]; then
  # Escape backslashes then double quotes so the url can never break out of its
  # JSON string (the endpoint is fixed benchmark prose today, but a stray quote
  # would otherwise emit invalid JSON and silently drop MCP).
  MCP_URL_ESC="${MCP_URL//\\/\\\\}"
  MCP_URL_ESC="${MCP_URL_ESC//\"/\\\"}"
  # `timeout` is opencode's per-fetch cap in ms (default 5000 — far below the
  # 300s a handbook tool call may take); match the servers' 300s tool timeout.
  MCP_JSON=",\"mcp\":{\"syntara\":{\"type\":\"remote\",\"url\":\"$MCP_URL_ESC\",\"enabled\":true,\"timeout\":300000}}"
fi

cat > "$HOME/.config/opencode/opencode.json" <<CONF
{"\$schema":"https://opencode.ai/config.json","model":"eval-containers/${MODEL:-default}","small_model":"eval-containers/${MODEL:-default}","disabled_providers":["anthropic","openai"],"provider":{"eval-containers":{"npm":"@ai-sdk/openai-compatible","name":"Eval Containers LiteLLM Proxy","options":{"baseURL":"${OPENAI_BASE_URL:-http://model:4000/v1}","apiKey":"${OPENAI_API_KEY:-sk-proxy}"},"models":{"${MODEL:-default}":{"name":"Eval Containers Proxy Model"}}}}${MCP_JSON},"permission":{"edit":"allow","bash":"allow","webfetch":"allow","external_directory":"allow"},"autoshare":false}
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1
# `--`: end-of-options. A task whose text begins with "-" is otherwise parsed as
# a flag — `opencode run` then prints its usage and exits 1 WITHOUT running, which
# scores as reward 0 and is indistinguishable from a wrong answer (observed on
# DeepSWE's ink-grid-box-layout, whose instruction.md opens with "- Update the
# `display` style property..."). Same class as the claude-code fix in 07b03181.
exec opencode run --model "eval-containers/${MODEL:-default}" -- "$TASK"
E

ENTRYPOINT ["/run.sh"]
