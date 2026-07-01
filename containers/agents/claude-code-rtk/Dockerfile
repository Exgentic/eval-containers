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
ARG AGENT_VERSION=2.1.104
# rtk's install.sh reads $RTK_VERSION (without the leading 'v') and infers
# the release URL. Build arg is named EVAL_RTK_VERSION to avoid shadowing
# the installer's internal env var; we pass it through explicitly below
# so the install pins to this exact release rather than 'latest'.
ARG EVAL_RTK_VERSION=v0.42.3
LABEL eval.type="agent"
LABEL eval.agent.name="claude-code-rtk"
LABEL eval.agent.description="Anthropic Claude Code CLI with rtk (Rust Token Killer) Bash output compression"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/anthropics/claude-code"
LABEL eval.agent.version="${AGENT_VERSION}"
LABEL eval.agent.rtk_version="${EVAL_RTK_VERSION}"
ENV EVAL_AGENT=claude-code-rtk

RUN npm install -g @anthropic-ai/claude-code@${AGENT_VERSION}

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
export CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1
export CLAUDE_CODE_MAX_OUTPUT_TOKENS="${CLAUDE_CODE_MAX_OUTPUT_TOKENS:-128000}"
# Register rtk's PreToolUse hook at runtime, when $HOME resolves to the
# agent user (not root). install.sh runs at combination-build time as
# root with $HOME=/root, which would write the hook where claude never
# reads it. `rtk init -g --auto-patch` writes ~/.claude/settings.json
# non-interactively.
mkdir -p "$HOME/.claude"
rtk init -g --auto-patch >>/output/agent/stderr.log 2>&1 || true
exec claude -p ${EVAL_AGENT_REASONING_EFFORT:+--effort $EVAL_AGENT_REASONING_EFFORT} --dangerously-skip-permissions "$TASK"
E

ENTRYPOINT ["/run.sh"]
