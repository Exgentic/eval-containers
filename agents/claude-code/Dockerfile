# syntax=docker/dockerfile:1
# Claude Code agent image (Anthropic)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code"
LABEL eval.agent.description="Anthropic Claude Code CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="2.1.104"
ENV EVAL_AGENT_VERSION_DEFAULT="2.1.104"
ENV EVAL_AGENT=claude-code
RUN npm install -g @anthropic-ai/claude-code@2.1.104
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx claude; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /eval-reinstall-agent <<'R' && chmod +x /eval-reinstall-agent
#!/bin/bash
set -euo pipefail
"$(command -v npm || echo /opt/agent/node/bin/npm)" install -g "@anthropic-ai/claude-code@${1:?version required}"
R
RUN mkdir -p /agent && cat > /agent/run.sh <<'E' && chmod +x /agent/run.sh
#!/bin/bash
set -euo pipefail
export ANTHROPIC_BASE_URL="${ANTHROPIC_BASE_URL:-http://model:4000}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1
export CLAUDE_CODE_MAX_OUTPUT_TOKENS="${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-128000}"
exec claude -p --dangerously-skip-permissions "$TASK"
E
ENTRYPOINT ["/agent/run.sh"]
