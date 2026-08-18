# syntax=docker/dockerfile:1
# Cline agent image (cline.bot)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.15.0
LABEL eval.type="agent"
LABEL eval.agent.name="cline"
LABEL eval.agent.description="Cline autonomous coding agent CLI (Plan/Act modes, MCP integration)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/cline/cline"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=cline
RUN npm install -g cline@${AGENT_VERSION}
RUN echo true > /opt/agent/MCP
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
cline auth -p openai -k "$OPENAI_API_KEY" -m "${EVAL_MODEL:-default}" -b "$OPENAI_BASE_URL" >/dev/null 2>&1 || true
# Benchmark-declared MCP servers (doctrine/agents rule 19). Like auth
# above, cline's MCP registry (~/.cline/data/settings/cline_mcp_settings.json)
# is only recognised when written through the CLI, so we register one
# server per entry rather than authoring the file. `-y` already
# auto-approves `use_mcp_tool`, so no extra permission wiring is needed.
# Unset or `{}` registers nothing.
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  while IFS=$'\t' read -r name url; do
    cline mcp add "$name" "$url" --type http >/dev/null 2>&1 || true
  done < <(node -e 'for (const [n, u] of Object.entries(JSON.parse(process.env.EVAL_MCP_SERVERS))) console.log(`${n}\t${u}`)')
fi
exec cline -y "$TASK"
E

ENTRYPOINT ["/run.sh"]
