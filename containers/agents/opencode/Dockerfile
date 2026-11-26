# syntax=docker/dockerfile:1
# opencode agent image (SST). v2: npm @opencode/cli, a native binary.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.4.1
LABEL eval.type="agent"
LABEL eval.agent.name="opencode"
LABEL eval.agent.description="SST opencode open-source AI coding agent"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/sst/opencode"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=opencode
# The package's postinstall would copy one of its platform builds over
# bin/opencode.exe, picked for the build host: the AVX2 x64 build on an AVX2 host,
# which crashes on a node without AVX2. So scripts are off and the image places
# the build that runs on any CPU of its arch itself (x64 baseline, glibc like the
# node it ships with), then drops the others, ~260 MB each.
RUN npm install -g --ignore-scripts @opencode/cli@${AGENT_VERSION} \
 && C=/opt/agent/node/lib/node_modules/@opencode/cli \
 && case "$(uname -m)" in x86_64) P=cli-linux-x64-baseline ;; aarch64) P=cli-linux-arm64 ;; *) exit 1 ;; esac \
 && rm -f "$C/bin/opencode.exe" && cp "$C/node_modules/@opencode/$P/bin/opencode" "$C/bin/opencode.exe" \
 && rm -rf "$C"/node_modules/@opencode/cli-* "$(npm config get cache)" \
 && opencode --version
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
for b in node npm npx opencode; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
# opencode's search tools use a system `rg`; with none they download ripgrep at
# first use, which a run with no internet cannot do. Best-effort: install.sh must
# work on any base (rule 11), offline or without apt-get.
if ! command -v rg >/dev/null 2>&1 && command -v apt-get >/dev/null 2>&1; then
  { apt-get update && apt-get install -y --no-install-recommends ripgrep; } \
    || echo "install.sh: no ripgrep to install; opencode will have to download it at run time" >&2
  rm -rf /var/lib/apt/lists/*
fi
I

# opencode's shell tool moves a command still running at its timeout (120 s
# unless the model sets one) to the background with no timeout, and `opencode
# run` waits for every background job before it exits, so a command that never
# returns would hold the task until EVAL_TIMEOUT (#744, #785). opencode has no
# setting for this, so its `shell` (config, /run.sh) is this wrapper: every
# command runs under timeout(1), which ends its process group at 600 s, Claude
# Code's maximum. The model is told the command timed out and the run goes on. A
# longer timeout the model asks for is cut at 600 s too, and so is a server it
# starts in the background.
RUN mkdir -p /opt/agent/bin && cat > /opt/agent/bin/bash <<'W' && chmod +x /opt/agent/bin/bash
#!/bin/bash
command -v timeout >/dev/null 2>&1 || exec /bin/bash "$@"
exec timeout -k 10 600 /bin/bash "$@"
W

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
  # `timeout` caps each MCP request in ms (connect, tool listing, each tool
  # call); match the servers' 300s tool timeout. `codemode:false` keeps the tools
  # direct: opencode 2 otherwise hides an MCP server's tools behind one `execute`
  # tool the model writes code for.
  MCP_JSON=",\"mcp\":{\"syntara\":{\"type\":\"remote\",\"url\":\"$MCP_URL_ESC\",\"enabled\":true,\"timeout\":300000,\"codemode\":false}}"
fi

# opencode's reply cap is 32000, and a turn cut at it ends the run, so it is
# raised to OPENCODE_MAX_OUTPUT_TOKENS (128000 unless the launch's agent env sets
# it; not the upstream name, as run-agent refuses any *TOKEN_* name there), as the
# model's `limit.output`. A non-numeric value means 128000. Told the model's window
# (EVAL_MODEL_CONTEXT_WINDOW, agents rules 25–26), the cap becomes min(that,
# window/4), 32768 on a 131k model where 128000 rejects the first call, and the
# window is the model's `limit.context`, which opencode compacts by (at window −
# max(15%, 16k)). Untold, `limit.context` is 0, a window opencode does not know:
# it does not compact ahead, as the 1.x line here never did, and compacts when the
# provider rejects a call as too long. Either way, a recovered overflow exits 0.
CAP="${OPENCODE_MAX_OUTPUT_TOKENS:-128000}"
[[ "$CAP" =~ ^[1-9][0-9]*$ ]] || CAP=128000
CTX=0
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  CTX=$EVAL_MODEL_CONTEXT_WINDOW
  CAP=$(( CAP < CTX / 4 ? CAP : CTX / 4 ))
fi
# `websearch` is denied: with no search provider configured, opencode asks the
# user to pick one, and an unattended run fails on the question (exit 1).
cat > "$HOME/.config/opencode/opencode.json" <<CONF
{"\$schema":"https://opencode.ai/config.json","model":"eval-containers/${MODEL:-default}","small_model":"eval-containers/${MODEL:-default}","disabled_providers":["anthropic","openai"],"shell":"/opt/agent/bin/bash","provider":{"eval-containers":{"npm":"@ai-sdk/openai-compatible","name":"Eval Containers LiteLLM Proxy","options":{"baseURL":"${OPENAI_BASE_URL:-http://model:4000/v1}","apiKey":"${OPENAI_API_KEY:-sk-proxy}"},"models":{"${MODEL:-default}":{"name":"Eval Containers Proxy Model","limit":{"context":${CTX},"output":${CAP}}}}}}${MCP_JSON},"permission":{"edit":"allow","bash":"allow","webfetch":"allow","external_directory":"allow","websearch":"deny"}}
CONF
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1
# `--standalone`: the run gets a private server that ends with it; without it,
# opencode reuses or starts a shared background service that outlives the run.
# stdin is /dev/null: `run` reads a stdin pipe to EOF, so an open one would hang.
# `--`: end-of-options. A task whose text begins with "-" is otherwise parsed as a
# flag, and `opencode run` exits without running, which scores as reward 0
# (observed on DeepSWE's ink-grid-box-layout, whose instruction.md opens with
# "- Update the `display` style property...").
exec opencode run --standalone --model "eval-containers/${MODEL:-default}" -- "$TASK" </dev/null
E

ENTRYPOINT ["/run.sh"]
