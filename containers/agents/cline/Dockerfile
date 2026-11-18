# syntax=docker/dockerfile:1
# Cline agent image (cline.bot)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.15.0
LABEL eval.type="agent"
LABEL eval.agent.name="cline"
LABEL eval.agent.description="Cline autonomous coding agent CLI (Plan/Act modes, MCP integration)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/cline/cline"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=cline
RUN npm install -g cline@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx cline; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# cline's auth state lives in $HOME/.cline/data/{globalState,secrets}.json
# and is only recognised when written via `cline auth`.
cline auth -p openai -k "$OPENAI_API_KEY" -m "${MODEL:-default}" -b "$OPENAI_BASE_URL" >/dev/null 2>&1 || true
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) is the model's contextWindow. With
# none set, cline 2.15.0 uses its default model info (128000, no reply cap); this
# writes that same info with the real window, for both modes. cline keeps its own
# truncation: before each request, once the last one used more than the window
# less 27k (64k window), 30k (128k), 40k (200k), or else max(window - 40k, 80%).
# It sends no reply cap, so nothing is reserved.
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  node -e 'const f=process.argv[1], s=require(f);
s.actModeOpenAiModelInfo = s.planModeOpenAiModelInfo = {maxTokens: -1, contextWindow: +process.argv[2],
  supportsImages: true, supportsPromptCache: false, isR1FormatRequired: false, inputPrice: 0, outputPrice: 0};
require("fs").writeFileSync(f, JSON.stringify(s))' "$HOME/.cline/data/globalState.json" "$EVAL_MODEL_CONTEXT_WINDOW"
fi
exec cline -y ${EVAL_AGENT_REASONING_EFFORT:+--reasoning-effort $EVAL_AGENT_REASONING_EFFORT} "$TASK"
E

ENTRYPOINT ["/run.sh"]
