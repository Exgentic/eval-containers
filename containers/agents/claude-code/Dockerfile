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
# Run as the run's model, as a user would with --model, so Claude Code applies
# what it knows of it. MODEL is the run's model as the agent sees it; the edge
# pins every call to the run's model whatever name goes out, so the name changes
# only what Claude Code assumes. A Claude model (one Claude Code recognises: an
# alias, or an ID holding a Claude family's name, which the docs say it resolves
# however a provider prefixes it) gets its own window, reply cap, effort levels and
# system prompt, sent as its Anthropic ID (the part from "claude-"). Any other
# model is sent by its name and is one Claude Code doesn't know. compose gives
# every agent the neutral eval-model (containers/compose/runner.yaml), which names
# no model: then, as with no MODEL, Claude Code runs as its default (Opus 5.5) and
# the real model is unknown. A launch's ANTHROPIC_MODEL wins (#765).
claude_knows() {
  case "${1%%\[*}" in
    opus | sonnet | haiku | fable | best | opusplan | default) return 0 ;;
    *claude-opus* | *claude-sonnet* | *claude-haiku* | *claude-fable* | *claude-[0-9]*) return 0 ;;
  esac
  return 1
}
case "${MODEL:-}" in
  "" | eval-model) ;;
  *)
    _m=$MODEL
    if claude_knows "$MODEL"; then case "$MODEL" in *claude-*) _m="claude-${MODEL#*claude-}" ;; esac; fi
    export ANTHROPIC_MODEL="${ANTHROPIC_MODEL:-$_m}"
    ;;
esac
# CC_KNOWS: Claude Code recognises the model it runs as (its default included).
# REAL_KNOWN: that model is the run's real one, so Claude Code knows its window.
CC_KNOWS=1 REAL_KNOWN=0
if [ -n "${ANTHROPIC_MODEL:-}" ]; then
  if claude_knows "$ANTHROPIC_MODEL"; then REAL_KNOWN=1; else CC_KNOWS=0; fi
fi
# Claude Code sends EVAL_AGENT_REASONING_EFFORT as is for a model it doesn't know,
# but lowers it, silently, to what a Claude model supports (Sonnet 4.6's xhigh
# goes out as high). Say so, so a comparison can't be skewed unseen; the level sent
# is in the call record (#765).
if [ -n "${EVAL_AGENT_REASONING_EFFORT:-}" ] && [ "$REAL_KNOWN" = 1 ]; then
  echo "claude-code: EVAL_AGENT_REASONING_EFFORT=$EVAL_AGENT_REASONING_EFFORT — Claude Code lowers a level $ANTHROPIC_MODEL does not support; the level sent is in the call record" >&2
fi
# Untold, a model Claude Code knows keeps its own window and reply cap. For any
# other it would assume 1M (and its default, Opus 5.5, would ask for 128000 per
# reply), so a request is refused once prompt + cap passes a smaller real window,
# before it compacts: the #726 failure on a 262k model. So it runs as 2.1.104 did:
# 1M off (CLAUDE_CODE_DISABLE_1M_CONTEXT, Claude Code's own setting: 200k, ~167k
# to compaction) and 32000 per reply. A launch's values win (#765).
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) tells Claude Code the model's real
# window. Claude Code compacts at window - min(reply cap, 20k) - 13k. A model it
# doesn't know is given the window as CLAUDE_CODE_MAX_CONTEXT_TOKENS, Anthropic's
# setting for a gateway or custom model (a Claude model ignores it). A model it
# knows keeps the window it knows for it, which the told one can only lower:
# CLAUDE_CODE_AUTO_COMPACT_WINDOW can, never below 100k, and under 100k
# CLAUDE_AUTOCOMPACT_PCT_OVERRIDE lowers the point instead (that share assumes the
# 100k window, so for a model given its window it is the share of that window,
# where the 100k share made a 64k window compact on every request). A launch's
# lower compaction window or share is kept, as the cap is: it only compacts
# sooner; a higher one is lowered to fit. At a cap up to 20000 the window is passed
# as is, so Claude Code keeps its own rule (98k on a 131k model, 967k on a 1M one). The reply cap becomes
# min(the launch's CLAUDE_CODE_MAX_OUTPUT_TOKENS or 32000, window/4): 32000 is what
# Claude Code sends a model it doesn't know, and a model that thinks for its whole
# reply needs it (a 20000 cap failed runs on such models, and Opus 4.8 overran it).
# Since 2.1.329 the compaction request is the whole context plus its prompt (~1.6k)
# at the full cap, and compaction reserves only min(cap, 20k), so at a 32000 cap
# point + cap left 1k and a 150k window was overrun on the compaction request
# itself (#765). So a cap past 20000 comes off the compaction window, C = window -
# (cap - 20000): point + cap is window - 13k whatever the cap, as at 20000. Under
# 100k C is set as the share above that puts the point in the same place.
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
  OUT=${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-32000}
  [[ "$OUT" =~ ^[1-9][0-9]*$ ]] || OUT=32000
  OUT=$(( OUT < W / 4 ? OUT : W / 4 ))
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS=$OUT
  # The compaction window C and the cap's share of it, RES (at most 20k).
  RES=$(( OUT < 20000 ? OUT : 20000 ))
  C=$(( W - OUT + RES ))
  if [ "$CC_KNOWS" = 1 ]; then
    if [ "$C" -ge 100000 ]; then
      AUTO=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$C}
      [[ "$AUTO" =~ ^[1-9][0-9]*$ && "$AUTO" -le "$C" ]] || AUTO=$C
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$AUTO
    else
      PCT=$(( (W - OUT - 13000) * 100 / (100000 - RES) ))
      SET=${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-$PCT}
      [[ "$SET" =~ ^[1-9][0-9]*$ && "$SET" -le "$PCT" ]] || SET=$PCT
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000 CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$SET
    fi
  else
    export CLAUDE_CODE_MAX_CONTEXT_TOKENS=$W
    # A: C, or under 100k, where C can't go, the window, with a share below.
    A=$(( C < 100000 ? W : C ))
    if [ -n "${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-}" ] || [ "$A" -lt "$W" ]; then
      AUTO=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$A}
      [[ "$AUTO" =~ ^[1-9][0-9]*$ && "$AUTO" -le "$A" ]] || AUTO=$A
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$AUTO
    fi
    if [ -n "${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-}" ] || [ "$A" -gt "$C" ]; then
      PCT=$(( (W - OUT - 13000) * 100 / (W - RES) ))
      SET=${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-$PCT}
      [[ "$SET" =~ ^[1-9][0-9]*$ && "$SET" -le "$PCT" ]] || SET=$PCT
      export CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$SET
    fi
  fi
elif [ "$REAL_KNOWN" = 0 ]; then
  export CLAUDE_CODE_DISABLE_1M_CONTEXT="${CLAUDE_CODE_DISABLE_1M_CONTEXT:-1}"
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS="${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-32000}"
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
