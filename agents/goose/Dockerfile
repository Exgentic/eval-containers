# syntax=docker/dockerfile:1
# Goose agent image (Block)
ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/agent-base-rust:latest
LABEL eval.type="agent"
LABEL eval.agent.name="goose"
LABEL eval.agent.description="Block Goose AI agent"
LABEL eval.agent.runtime="rust"
LABEL eval.agent.url="https://github.com/block/goose"
LABEL eval.agent.version="1.30.0"
ENV EVAL_AGENT_VERSION_DEFAULT="1.30.0"
ENV EVAL_AGENT=goose
# Download + integrity-check, then extract. Streaming `curl | tar` fails
# on partial bytes under flaky networks (observed 2026-04-17); download
# to a file so curl can retry and we can size-check before extracting.
RUN set -e; ok=0; ARCH=$(uname -m); \
    URL="https://github.com/block/goose/releases/download/v1.30.0/goose-${ARCH}-unknown-linux-gnu.tar.bz2"; \
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
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export GOOSE_PROVIDER="openai"
export GOOSE_MODEL="${EVAL_MODEL:?EVAL_MODEL must be set}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_HOST="${OPENAI_BASE_URL:-http://model:4000}"
export GOOSE_TELEMETRY_OFF=true
exec goose run -t "$TASK" --no-session -q
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
