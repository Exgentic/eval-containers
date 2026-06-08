# syntax=docker/dockerfile:1
# OpenClaw agent image
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
LABEL eval.type="agent"
LABEL eval.agent.name="openclaw"
LABEL eval.agent.description="OpenClaw general-purpose AI agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/openclaw/openclaw"
LABEL eval.agent.version="2026.4.11"
ENV EVAL_AGENT_VERSION_DEFAULT="2026.4.11"
ENV EVAL_AGENT=openclaw
RUN npm install -g openclaw@2026.4.11
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx openclaw; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I
RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
mkdir -p ~/.openclaw
cat > ~/.openclaw/openclaw.json <<CONF
{"models":{"providers":{"eval-containers":{"baseUrl":"${OPENAI_BASE_URL:-http://model:4000}","apiKey":"${OPENAI_API_KEY:-sk-proxy}","api":"openai-completions","models":[{"id":"eval-model","name":"Eval Containers Proxy Model"}]}}},"agents":{"defaults":{"model":{"primary":"eval-containers/eval-model"}}},"tools":{"exec":{"security":"full","ask":"off"}}}
CONF
cat > ~/.openclaw/exec-approvals.json <<CONF
{"version":1,"defaults":{"security":"full","ask":"off","askFallback":"full"},"agents":{"*":{"security":"full","ask":"off","askFallback":"full","allowlist":[{"pattern":"**"}]}}}
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# openclaw 2026.4.11 requires a session selector (one of --to /
# --session-id / --agent). --session-id is the cheapest — picks an
# ad-hoc session derived from the literal string. No real session
# state is needed for non-interactive agent invocations.
exec openclaw agent --local --session-id smoke --message "$TASK"
E
ENTRYPOINT ["/run.sh"]
