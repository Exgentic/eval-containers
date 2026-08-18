# syntax=docker/dockerfile:1
# OpenClaw agent image
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2026.4.11
LABEL eval.type="agent"
LABEL eval.agent.name="openclaw"
LABEL eval.agent.description="OpenClaw general-purpose AI agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/openclaw/openclaw"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=openclaw
RUN npm install -g openclaw@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
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
# Benchmark-declared MCP servers (doctrine/agents rule 19). openclaw reads
# them from `mcp.servers` in the same openclaw.json written above, so we
# merge in rather than rewrite — the provider block is what lets the agent
# reach the gateway at all. `transport` is mandatory here: omitting it
# defaults to SSE, which our streamable-HTTP servers answer with 405.
# Unset or `{}` leaves the file exactly as the heredoc wrote it.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  node -e 'const fs = require("fs"), p = process.env.HOME + "/.openclaw/openclaw.json"; const c = JSON.parse(fs.readFileSync(p, "utf8")); c.mcp = {servers: Object.fromEntries(Object.entries(JSON.parse(process.env.EVAL_MCP_SERVERS)).map(([n, u]) => [n, {url: u, transport: "streamable-http", enabled: true}]))}; fs.writeFileSync(p, JSON.stringify(c))'
fi
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# openclaw ${AGENT_VERSION} requires a session selector (one of --to /
# --session-id / --agent). --session-id is the cheapest — picks an
# ad-hoc session derived from the literal string. No real session
# state is needed for non-interactive agent invocations.
exec openclaw agent --local --session-id smoke --message "$TASK"
E

ENTRYPOINT ["/run.sh"]
