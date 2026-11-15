# syntax=docker/dockerfile:1
# Goose agent image (Block)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
ARG AGENT_VERSION=1.30.0
LABEL eval.type="agent"
LABEL eval.agent.name="goose"
LABEL eval.agent.description="Block Goose AI agent"
LABEL eval.agent.runtime="rust"
LABEL eval.agent.url="https://github.com/block/goose"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=goose
# Download + integrity-check, then extract. Streaming `curl | tar` fails
# on partial bytes under flaky networks (observed 2026-04-17); download
# to a file so curl can retry and we can size-check before extracting.
RUN set -e; ok=0; ARCH=$(uname -m); \
    URL="https://github.com/block/goose/releases/download/v${AGENT_VERSION}/goose-${ARCH}-unknown-linux-gnu.tar.bz2"; \
    for i in 1 2 3 4 5; do \
      if curl -fsSL --retry 5 --retry-delay 3 --retry-connrefused --max-time 300 \
           "$URL" -o /tmp/goose.tar.bz2 \
         && [ "$(stat -c %s /tmp/goose.tar.bz2)" -gt 1000000 ]; then \
        ok=1; break; \
      fi; \
      echo "goose tarball attempt $i failed; retry" >&2; rm -f /tmp/goose.tar.bz2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "goose tarball download failed" >&2; exit 1; }; \
    tar xj -C /opt/agent/bin --strip-components=0 ./goose -f /tmp/goose.tar.bz2 && \
    rm /tmp/goose.tar.bz2 && \
    chmod +x /opt/agent/bin/goose && ln -sf /opt/agent/bin/goose /usr/local/bin/goose
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
[ -x /opt/agent/bin/goose ] && ln -sf /opt/agent/bin/goose /usr/local/bin/goose || true
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export GOOSE_PROVIDER="openai"
export GOOSE_MODEL="${MODEL:-default}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# Goose appends /v1/chat/completions to OPENAI_HOST, but rule 5
# delivers OPENAI_BASE_URL with /v1 baked in. Strip the suffix so we
# don't end up posting to /openai/v1/v1/chat/completions.
H="${OPENAI_BASE_URL:-http://model:4000}"; export OPENAI_HOST="${H%/v1}"
export GOOSE_TELEMETRY_OFF=true
# EVAL_MODEL_CONTEXT (agents rules 25-26) becomes goose's context limit (128k
# unless told), and goose keeps its own compaction: at 80% of the limit. In 1.30.0
# `goose run` checks that point once, before the first request; during the task it
# compacts only when the server rejects a call as too long, then retries (giving
# up on a second rejection in a row). The reply cap, GOOSE_MAX_TOKENS, becomes
# min(32000, window/5), so 80% plus a reply fits; it replaces both the 4096 goose
# uses for a model it doesn't know and a catalog model's own cap. A hosted model
# whose output limit is below that cap rejects it with a 400 that goose reads as
# an overflow, so such a model should not be told the window. Under 20480 the cap
# would be under 4096, and the window fails before the first call.
if [ -n "${EVAL_MODEL_CONTEXT:-}" ]; then
  W=$EVAL_MODEL_CONTEXT
  if [ "$W" -lt 20480 ]; then
    echo "goose: EVAL_MODEL_CONTEXT=$W is too small to hold a reply and a compacted prompt" >&2
    exit 2
  fi
  export GOOSE_CONTEXT_LIMIT=$W GOOSE_MAX_TOKENS=$(( W / 5 < 32000 ? W / 5 : 32000 ))
fi
exec goose run -t "$TASK" --no-session -q
E

ENTRYPOINT ["/run.sh"]
