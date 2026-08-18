# syntax=docker/dockerfile:1
# Claude Code agent image (Anthropic)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.1.104
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code"
LABEL eval.agent.description="Anthropic Claude Code CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19): this agent renders
# EVAL_MCP_SERVERS into its own config. The label is for discovery
# (`docker inspect`, image listings); /opt/agent/MCP below is what the
# runtime guard reads, because labels do not survive the combination
# build (the eval image inherits the *benchmark* image's labels).
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=claude-code
RUN npm install -g @anthropic-ai/claude-code@${AGENT_VERSION}
# Same publish-a-fact mechanism as /opt/agent/VERSION: combination.Dockerfile
# copies all of /opt/agent/, so this is the one place the assembled eval
# image can learn what the agent supports.
RUN echo true > /opt/agent/MCP
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

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export ANTHROPIC_BASE_URL="${ANTHROPIC_BASE_URL:-http://model:4000}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1
export CLAUDE_CODE_MAX_OUTPUT_TOKENS="${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-128000}"
# Benchmark-declared MCP servers (doctrine/agents rule 19). The benchmark
# publishes a {name: address} map; we render it into claude's own
# {"mcpServers": {name: {"type": "http", "url": ...}}} shape. Every entry
# becomes a server, so a benchmark can stand up as many sidecars as it
# likes and this script never changes.
#
# `type` is mandatory at this pin — an entry with a bare `url` and no
# `type` is skipped without a diagnostic. `--strict-mcp-config` makes
# this the *only* source of MCP config so a stray ~/.claude.json in some
# benchmark base can neither add servers nor shadow ours.
#
# node, not python3: `node` is guaranteed present here (install.sh
# symlinks it from /opt/agent/node, which the combination image copies),
# whereas the interpreter set of an arbitrary benchmark base is not ours
# to assume — rule 11.
#
# Unset or `{}` leaves the command line byte-identical to the no-MCP
# case, so existing replay fixtures keep matching.
MCP=()
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  MCP=(--strict-mcp-config --mcp-config "$(node -e 'const m = JSON.parse(process.env.EVAL_MCP_SERVERS); console.log(JSON.stringify({mcpServers: Object.fromEntries(Object.entries(m).map(([n, u]) => [n, {type: "http", url: u}]))}))')")
fi
# $TASK goes BEFORE the MCP flags, and --mcp-config is last. It is
# variadic (it accepts a list of config paths/objects), so anything
# after it is eaten as another config entry — with the flags first,
# claude read the prompt as a filename and died with
# "MCP config file not found: /app/Reply with exactly: OK".
exec claude -p --dangerously-skip-permissions "$TASK" "${MCP[@]}"
E

ENTRYPOINT ["/run.sh"]
