# syntax=docker/dockerfile:1
# Gemini CLI agent image (Google)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.37.2
LABEL eval.type="agent"
LABEL eval.agent.name="gemini-cli"
LABEL eval.agent.description="Google Gemini CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/google-gemini/gemini-cli"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=gemini-cli
RUN npm install -g @google/gemini-cli@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx gemini; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export GOOGLE_GEMINI_BASE_URL="${GOOGLE_GEMINI_BASE_URL:-http://model:4000}"
export GEMINI_API_KEY="${GEMINI_API_KEY:-sk-proxy}"
export GEMINI_SANDBOX=false
# Benchmark-declared MCP servers (doctrine/agents rule 19). gemini-cli
# has no flag for this — servers come from `mcpServers` in
# settings.json, so we author the file. `type: "http"` selects
# streamable HTTP (the older `httpUrl` key is deprecated at this pin)
# and `trust: true` skips the per-tool confirmation that `--yolo` does
# not cover for MCP. Unset or `{}` writes no file at all, leaving the
# invocation byte-identical to the no-MCP case.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  mkdir -p "$HOME/.gemini"
  node -e 'const m = JSON.parse(process.env.EVAL_MCP_SERVERS); const s = {mcpServers: Object.fromEntries(Object.entries(m).map(([n, u]) => [n, {type: "http", url: u, trust: true}]))}; require("fs").writeFileSync(process.env.HOME + "/.gemini/settings.json", JSON.stringify(s))'
fi
exec gemini --yolo -p "$TASK"
E

ENTRYPOINT ["/run.sh"]
