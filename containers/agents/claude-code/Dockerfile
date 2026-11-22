# syntax=docker/dockerfile:1
# Claude Code agent image (Anthropic)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.1.329
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code"
LABEL eval.agent.description="Anthropic Claude Code CLI"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=claude-code
# The CLI is a native binary its postinstall places from an optional dependency;
# a failed fetch leaves a stub that exits 1, so prove it runs here.
RUN npm install -g @anthropic-ai/claude-code@${AGENT_VERSION} && claude --version \
 && echo "${AGENT_VERSION}" > /opt/agent/VERSION
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
# A launch may set it (0 lets Claude Code send its experimental beta headers);
# unset, they stay off, for a proxy that may not support them.
export CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS="${CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS:-1}"
# Run as the run's model, as a user would with --model, so Claude Code applies
# what it knows of it: a Claude model gets its own window, reply cap, effort
# levels and system prompt (sent as its Anthropic ID, the part from "claude-",
# which Claude Code recognises however a provider prefixes it); any other model
# is one it doesn't know, told its window below. MODEL is the run's model as the
# agent sees it; the edge pins every call to the run's model whatever name goes
# out, so the name changes only what Claude Code assumes. compose gives every
# agent the neutral eval-model (containers/compose/runner.yaml), which names no
# model: then, as with no MODEL, Claude Code keeps its default. A launch's
# ANTHROPIC_MODEL wins (#765).
case "${MODEL:-}" in
  "" | eval-model) ;;
  *claude-*) export ANTHROPIC_MODEL="${ANTHROPIC_MODEL:-claude-${MODEL#*claude-}}" ;;
  *) export ANTHROPIC_MODEL="${ANTHROPIC_MODEL:-$MODEL}" ;;
esac
# No CLAUDE_CODE_MAX_OUTPUT_TOKENS by default (EVAL_MODEL_CONTEXT_WINDOW below sets
# one to fit the window it is told): Claude Code asks for its own per-model reply
# cap and resumes a reply cut there. Untold, it assumes the window it knows for a
# Claude model, and 1M for any other (32000 per reply; 128000 for its default,
# Opus 5.5), so a request is refused once prompt + cap passes a smaller real
# window; the launcher tells the window (#765).
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) tells Claude Code the model's real
# window. Claude Code compacts at window - min(reply cap, 20k) - 13k. A model it
# doesn't know is given the window as CLAUDE_CODE_MAX_CONTEXT_TOKENS, Anthropic's
# setting for a gateway or custom model (a Claude model ignores it). A Claude
# model keeps the window Claude Code knows for it, which the told one can only
# lower: CLAUDE_CODE_AUTO_COMPACT_WINDOW can, never below 100k, and under 100k
# CLAUDE_AUTOCOMPACT_PCT_OVERRIDE lowers the point instead (that share assumes the
# 100k window, so it is not set beside CLAUDE_CODE_MAX_CONTEXT_TOKENS, where a 64k
# window compacted on every request). The window is passed as is, so Claude Code
# keeps its own rule (98k on a 131k model, 967k on a 1M one). The reply cap becomes
# min(the launch's CLAUDE_CODE_MAX_OUTPUT_TOKENS, 20000, window/4), so point + cap
# is window - 13k and a lower cap set for a model's output limit is kept. Not
# 32000: since 2.1.329 the compaction request is the whole context plus its
# prompt (~1.6k) at the full cap, and with a 32000 cap point + cap left 1k, so a
# 150k window was overrun on the compaction request itself (#765).
# Below 64000 it fails before the first call: the first request (system prompt
# and tools) is ~22k tokens and the first after a compaction ~25k, and the
# compaction point must clear that by a few turns or it compacts on every
# request (~34k at 64k, the tests/run/context window).
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  W=$EVAL_MODEL_CONTEXT_WINDOW
  if [ "$W" -lt 64000 ]; then
    echo "claude-code: EVAL_MODEL_CONTEXT_WINDOW=$W is below 64000, the smallest window it fits (tests/run/context)" >&2
    exit 2
  fi
  OUT=${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-20000}
  [[ "$OUT" =~ ^[1-9][0-9]*$ && "$OUT" -le 20000 ]] || OUT=20000
  OUT=$(( OUT < W / 4 ? OUT : W / 4 ))
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS=$OUT
  case "${ANTHROPIC_MODEL:-claude-}" in
    claude-*)
      # A model Claude Code knows (its default included) keeps its own window; the
      # told one can only lower it. A lower compaction window or share the launch
      # set is kept, as the cap is: it only compacts sooner. A higher one is
      # lowered to fit.
      if [ "$W" -ge 100000 ]; then
        AUTO=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$W}
        [[ "$AUTO" =~ ^[1-9][0-9]*$ && "$AUTO" -le "$W" ]] || AUTO=$W
        export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$AUTO
      else
        PCT=$(( (W - OUT - 13000) * 100 / (100000 - OUT) ))
        SET=${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-$PCT}
        [[ "$SET" =~ ^[1-9][0-9]*$ && "$SET" -le "$PCT" ]] || SET=$PCT
        export CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000 CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$SET
      fi
      ;;
    *)
      # One it doesn't know is given the window, and compacts by its own rule.
      export CLAUDE_CODE_MAX_CONTEXT_TOKENS=$W
      ;;
  esac
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
