# syntax=docker/dockerfile:1
# Claude Code agent image (Anthropic)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.1.104
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code"
LABEL eval.agent.description="Anthropic Claude Code CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=claude-code
RUN npm install -g @anthropic-ai/claude-code@${AGENT_VERSION}
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
# No CLAUDE_CODE_MAX_OUTPUT_TOKENS by default (EVAL_MODEL_CONTEXT below sets
# one to fit the window it is told). With no --model, Claude Code runs as its
# default model, Sonnet 4.6, whatever model the gateway maps that name to, so
# it caps a reply at Sonnet 4.6's default, 32000, and resumes a reply cut there.
# Forcing 128000 reserved 128k of every model's context window, so on a
# 262k-window vLLM model each call failed once the prompt passed ~134k (#726).
# EVAL_MODEL_CONTEXT (agents rules 25-26) tells Claude Code the model's real
# window: it assumes Sonnet 4.6's 200k, so on a smaller model every call past the
# real window is rejected. Claude Code 2.1.104 compacts at its window minus
# min(reply cap, 20k) minus 13k; CLAUDE_CODE_AUTO_COMPACT_WINDOW can only lower
# that window (it takes the smaller of the two) and never below 100k, and under
# that CLAUDE_AUTOCOMPACT_PCT_OVERRIDE lowers the point as a percent of the window
# less the reply allowance. The window is passed as is, so Claude Code keeps its
# own rule (98k on a 131k model). The reply cap becomes min(32000, window/4),
# which keeps point + cap within the window, also for a model whose default cap
# is larger (64000 for opus-4-6). Above 200k nothing but the cap changes, as the
# window can't be raised.
if [ -n "${EVAL_MODEL_CONTEXT:-}" ]; then
  W=$EVAL_MODEL_CONTEXT
  OUT=$(( W / 4 < 32000 ? W / 4 : 32000 ))
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS=$OUT
  if [ "$W" -ge 100000 ]; then
    export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$W
  else
    ALLOW=$(( OUT < 20000 ? OUT : 20000 ))
    PCT=$(( (W - ALLOW - 13000) * 100 / (100000 - ALLOW) ))
    if [ "$PCT" -lt 1 ]; then
      echo "claude-code: EVAL_MODEL_CONTEXT=$W is too small to hold a reply and a compacted prompt" >&2
      exit 2
    fi
    export CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000 CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$PCT
  fi
fi
# Web tools are gated on the benchmark's declared internet policy: when the
# benchmark sets EVAL_INTERNET=false (mirrors its eval.benchmark.internet label),
# deny WebSearch/WebFetch so the fix comes from the agent, not a search — some
# task ids encode their upstream source, so a web fetch could leak the reference
# fix. Unset (the default) leaves web tools enabled, so the reproducible no-env
# run is unchanged (agents rule 12) and the flag stays a generic capability
# toggle, not benchmark knowledge (rule 4). Neither denial is the isolation
# boundary — benchmarks rule 9 already blocks the agent's raw internet access
# unconditionally on every benchmark and every surface, WebFetch included, so
# denying it just avoids a call bound to fail instead of a confusing failure.
# WebSearch is different: its search runs server-side through the LLM
# provider, not as a socket the agent opens, so it's the one channel network
# isolation can't reach — denying it client-side is currently the only lever
# there, and only a temporary mitigation (blocking it at the gateway is the
# real fix, tracked separately).
DENY_WEB=""
if [ "${EVAL_INTERNET:-}" = "false" ]; then DENY_WEB="--disallowed-tools WebSearch,WebFetch"; fi
# `--`: a task beginning with "--" is otherwise parsed as an option, and the
# agent exits 1 without running.
exec claude -p ${EVAL_AGENT_REASONING_EFFORT:+--effort $EVAL_AGENT_REASONING_EFFORT} ${DENY_WEB} --dangerously-skip-permissions -- "$TASK"
E

ENTRYPOINT ["/run.sh"]
