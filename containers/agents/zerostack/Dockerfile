# syntax=docker/dockerfile:1
# Zerostack agent image — minimal Rust coding agent (gi-dellav/zerostack)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
ARG AGENT_VERSION=1.5.0
LABEL eval.type="agent"
LABEL eval.agent.name="zerostack"
LABEL eval.agent.description="Zerostack minimal Rust coding agent"
LABEL eval.agent.runtime="rust"
LABEL eval.agent.url="https://github.com/gi-dellav/zerostack"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=zerostack
# Prebuilt release binary — the tarball is a single `zerostack` at its root.
# Download to a file + size-check before extracting; streaming `curl | tar`
# corrupts on partial bytes under flaky networks (same pattern as goose).
RUN set -e; ok=0; ARCH=$(uname -m); \
    URL="https://github.com/gi-dellav/zerostack/releases/download/v${AGENT_VERSION}/zerostack-${ARCH}-unknown-linux-gnu.tar.gz"; \
    for i in 1 2 3 4 5; do \
      if curl -fsSL --retry 5 --retry-delay 3 --retry-connrefused --max-time 300 \
           "$URL" -o /tmp/zerostack.tar.gz \
         && [ "$(stat -c %s /tmp/zerostack.tar.gz)" -gt 1000000 ]; then \
        ok=1; break; \
      fi; \
      echo "zerostack tarball attempt $i failed; retry" >&2; rm -f /tmp/zerostack.tar.gz; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "zerostack tarball download failed" >&2; exit 1; }; \
    tar xz -C /opt/agent/bin -f /tmp/zerostack.tar.gz && \
    rm /tmp/zerostack.tar.gz && \
    chmod +x /opt/agent/bin/zerostack && ln -sf /opt/agent/bin/zerostack /usr/local/bin/zerostack
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
[ -x /opt/agent/bin/zerostack ] && ln -sf /opt/agent/bin/zerostack /usr/local/bin/zerostack || true
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
# Route every model call through the gateway. Zerostack reads config from
# $ZS_CONFIG_DIR/config.json; we declare a custom OpenAI-compatible provider
# whose base_url is the framework's protocol-namespaced OpenAI endpoint. rig
# (zerostack's client) appends /chat/completions to the /v1 base, so pass
# OPENAI_BASE_URL through unmodified — no /v1 stripping (agents/RULES.md rule 5).
# api_style=completions: the gateway speaks Chat Completions, not the Responses
# API. enable-exa-mcp=false: the default-on Exa MCP would reach mcp.exa.ai, but
# the gateway is the only egress, so disable it.
export ZS_CONFIG_DIR=/tmp/zerostack
mkdir -p "$ZS_CONFIG_DIR"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# Name (not value) of the env var holding the placeholder key; the gateway holds
# the real upstream credential, so this is not a secret.
key_env=OPENAI_API_KEY  # pragma: allowlist secret
cat > "$ZS_CONFIG_DIR/config.json" <<CONF
{"custom_providers":{"eval":{"provider_type":"openai","base_url":"${OPENAI_BASE_URL:-http://gateway:4000/openai/v1}","api_key_env":"${key_env}","api_style":"completions"}},"enable-exa-mcp":false}
CONF
# -p prints the final answer to stdout and exits (no TUI). --no-session keeps the
# run ephemeral. --dangerously-skip-permissions: the container is the sandbox, so
# the agent runs unguarded and never blocks on a permission prompt (agents/RULES.md
# rule 10, no self-sandboxing). --no-context-files: skip the AGENTS.md/ARCHITECTURE.md
# scan, which otherwise prompts "Create one? [y/N]" and pollutes stdout when there is
# no TTY. `--` guards a task that starts with '-'.
# Model name comes from $MODEL: run-agent scrubs the env (`env -i`) and forwards
# only an allow-list — it maps EVAL_MODEL → MODEL, so the agent never sees
# EVAL_MODEL itself.
exec zerostack -p \
  --provider eval \
  --model "${MODEL:?MODEL must be set}" \
  --no-session \
  --dangerously-skip-permissions \
  --no-context-files \
  -- "$TASK"
E

ENTRYPOINT ["/run.sh"]
