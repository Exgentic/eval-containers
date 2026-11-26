# syntax=docker/dockerfile:1
# Claude Code agent + rtk (Rust Token Killer).
#
# rtk is a CLI proxy that registers a PreToolUse hook in Claude Code's
# settings.json and rewrites Bash tool calls (git/npm/cargo/pytest/...)
# so their output is compressed before it lands back in the agent's
# context window. Upstream claims 60-90% token reduction on shell-heavy
# tasks (https://github.com/rtk-ai/rtk).
#
# Identical to containers/agents/claude-code in every other respect —
# same upstream CLI pin, same /run.sh, same env contract. Two
# differences from the parent:
#   1) installs the rtk binary at /opt/agent/bin/rtk
#   2) /run.sh registers the rtk hook in $HOME/.claude/settings.json
#      at runtime (when $HOME = the agent user's home, not root's)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=2.1.329
# Named EVAL_RTK_VERSION, not RTK_VERSION, so it doesn't shadow the variable
# rtk's install.sh reads; the install below passes it through explicitly so it
# pins this exact release rather than 'latest'.
ARG EVAL_RTK_VERSION=v0.42.3
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code-rtk"
LABEL eval.agent.description="Anthropic Claude Code CLI with rtk (Rust Token Killer) Bash output compression"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="${AGENT_VERSION}"
LABEL eval.agent.rtk_version="${EVAL_RTK_VERSION}"
ENV EVAL_AGENT=claude-code-rtk

# The CLI is a native binary its postinstall places from an optional dependency;
# a failed fetch leaves a stub that exits 1, so prove it runs here.
RUN npm install -g @anthropic-ai/claude-code@${AGENT_VERSION} && claude --version \
 && echo "${AGENT_VERSION}" > /opt/agent/VERSION

# rtk install — single Rust binary. Lands under /opt/agent/bin so the
# combination image (which COPYs /opt/agent/ from this image) picks it
# up. Putting it in /usr/local/bin would NOT reach the combination,
# since core/combination.Dockerfile only copies /opt/agent/ + /run.sh.
# RTK_VERSION is passed through verbatim: install.sh puts it straight into the
# release-download URL, and rtk's tags carry the leading 'v' (…/download/v0.42.3/…),
# so stripping the 'v' 404s.
RUN mkdir -p /opt/agent/bin \
 && curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/refs/heads/master/install.sh \
      | RTK_VERSION="${EVAL_RTK_VERSION}" sh \
 && cp /root/.local/bin/rtk /opt/agent/bin/rtk \
 && /opt/agent/bin/rtk --version

RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
for b in node npm npx claude; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
ln -sf /opt/agent/bin/rtk /usr/local/bin/rtk
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
# Run as the run's model (MODEL) — see containers/agents/claude-code/Dockerfile.
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
# The reply cap and window, told or not — see containers/agents/claude-code/Dockerfile.
# Register rtk's PreToolUse hook at runtime, when $HOME resolves to the
# agent user (not root). install.sh runs at combination-build time as
# root with $HOME=/root, which would write the hook where claude never
# reads it. `rtk init -g --auto-patch` writes ~/.claude/settings.json
# non-interactively.
mkdir -p "$HOME/.claude"
rtk init -g --auto-patch >>/output/agent/stderr.log 2>&1 || true
# EVAL_MODEL_CONTEXT_WINDOW (agents rules 25-26) sizes Claude Code to the window it is
# told — see containers/agents/claude-code/Dockerfile for the derivation.
if [ -n "${EVAL_MODEL_CONTEXT_WINDOW:-}" ]; then
  W=$EVAL_MODEL_CONTEXT_WINDOW
  # A launch value is used only if it is 1-9 digits: a longer one wraps in bash
  # arithmetic (a cap of 2^63 goes negative) and is treated as unset.
  N='^[1-9][0-9]{0,8}$'
  if ! [[ "$W" =~ $N ]] || [ "$W" -lt 64000 ]; then
    echo "claude-code: EVAL_MODEL_CONTEXT_WINDOW=$W must be 64000 (the smallest window it fits, tests/run/context) to 999999999" >&2
    exit 2
  fi
  OUT=${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-32000}
  [[ "$OUT" =~ $N ]] || OUT=32000
  OUT=$(( OUT < W / 4 ? OUT : W / 4 ))
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS=$OUT
  # The compaction window C and the cap's share of it, RES (at most 20k).
  RES=$(( OUT < 20000 ? OUT : 20000 ))
  C=$(( W - OUT + RES ))
  if [ "$CC_KNOWS" = 1 ]; then
    if [ "$C" -ge 100000 ]; then
      AUTO=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$C}
      [[ "$AUTO" =~ $N && "$AUTO" -le "$C" ]] || AUTO=$C
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$AUTO
    else
      PCT=$(( (W - OUT - 13000) * 100 / (100000 - RES) ))
      SET=${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-$PCT}
      [[ "$SET" =~ $N && "$SET" -le "$PCT" ]] || SET=$PCT
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=100000 CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$SET
    fi
  else
    export CLAUDE_CODE_MAX_CONTEXT_TOKENS=$W
    # A: C, or under 100k, where C can't go, the window, with a share below.
    A=$(( C < 100000 ? W : C ))
    if [ -n "${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-}" ] || [ "$A" -lt "$W" ]; then
      AUTO=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$A}
      [[ "$AUTO" =~ $N && "$AUTO" -le "$A" ]] || AUTO=$A
      export CLAUDE_CODE_AUTO_COMPACT_WINDOW=$AUTO
    fi
    if [ -n "${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-}" ] || [ "$A" -gt "$C" ]; then
      # The share is of the window Claude Code compacts in, less RES: the one set
      # above (never below 100k), or the window itself. Past 100 it is ignored.
      E=${CLAUDE_CODE_AUTO_COMPACT_WINDOW:-$W}
      E=$(( E > 100000 ? E : 100000 )); E=$(( E < W ? E : W ))
      PCT=$(( (W - OUT - 13000) * 100 / (E - RES) )); PCT=$(( PCT < 100 ? PCT : 100 ))
      SET=${CLAUDE_AUTOCOMPACT_PCT_OVERRIDE:-$PCT}
      [[ "$SET" =~ $N && "$SET" -le "$PCT" ]] || SET=$PCT
      export CLAUDE_AUTOCOMPACT_PCT_OVERRIDE=$SET
    fi
  fi
elif [ "$REAL_KNOWN" = 0 ]; then
  export CLAUDE_CODE_DISABLE_1M_CONTEXT="${CLAUDE_CODE_DISABLE_1M_CONTEXT:-1}"
  export CLAUDE_CODE_MAX_OUTPUT_TOKENS="${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-32000}"
fi
# Web tools are gated on the benchmark's declared internet policy — see
# containers/agents/claude-code/Dockerfile for the full rationale.
DENY_WEB=""
if [ "${EVAL_INTERNET:-}" = "false" ]; then DENY_WEB="--disallowed-tools WebSearch,WebFetch"; fi
exec claude -p ${EVAL_AGENT_REASONING_EFFORT:+--effort $EVAL_AGENT_REASONING_EFFORT} ${DENY_WEB} --dangerously-skip-permissions -- "$TASK"
E

ENTRYPOINT ["/run.sh"]
