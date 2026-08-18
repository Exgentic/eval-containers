# syntax=docker/dockerfile:1
# Goose agent image (Block)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
ARG AGENT_VERSION=1.30.0
LABEL eval.type="agent"
LABEL eval.agent.name="goose"
LABEL eval.agent.description="Block Goose AI agent"
LABEL eval.agent.runtime="rust"
LABEL eval.agent.url="https://github.com/block/goose"
LABEL eval.agent.version="${AGENT_VERSION}"
# Declares the MCP contract (doctrine/agents rule 19). The label is for
# discovery only; /opt/agent/MCP below is what the runtime guard reads,
# because labels do not survive the combination build.
LABEL eval.agent.mcp="true"
ENV EVAL_AGENT=goose
RUN echo true > /opt/agent/MCP
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
export GOOSE_MODEL="${EVAL_MODEL:?EVAL_MODEL must be set}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
# Goose appends /v1/chat/completions to OPENAI_HOST, but rule 5
# delivers OPENAI_BASE_URL with /v1 baked in. Strip the suffix so we
# don't end up posting to /openai/v1/v1/chat/completions.
H="${OPENAI_BASE_URL:-http://model:4000}"; export OPENAI_HOST="${H%/v1}"
export GOOSE_TELEMETRY_OFF=true
# Benchmark-declared MCP servers (doctrine/agents rule 19). goose calls
# them "extensions" and takes one flag per server, so we build the flag
# list from the map — a benchmark can declare as many as it likes and
# this script never changes. Flags beat ~/.config/goose/config.yaml here:
# they can't be shadowed by a stale config in some benchmark base.
#
# awk, not node/python: goose is a static binary dropped onto whatever
# base the benchmark chose, and rule 11 says we don't get to assume that
# base ships an interpreter. awk is POSIX. The map is flat string→string,
# so splitting each comma-separated pair on its first `:` is enough —
# URLs contain `:` but neither `,` nor `"`.
MCP=()
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  while IFS="$(printf '\t')" read -r name url; do
    [ -n "$name" ] || continue
    MCP+=(--with-streamable-http-extension "$url")
  done < <(printf '%s' "$EVAL_MCP_SERVERS" | awk '{
    gsub(/[{}"]/, ""); n = split($0, e, ",");
    for (i = 1; i <= n; i++) {
      p = index(e[i], ":"); if (p == 0) continue;
      k = substr(e[i], 1, p - 1); v = substr(e[i], p + 1);
      gsub(/^[ \t]+|[ \t]+$/, "", k); gsub(/^[ \t]+|[ \t]+$/, "", v);
      if (k != "") printf "%s\t%s\n", k, v;
    }
  }')
fi
exec goose run -t "$TASK" --no-session -q "${MCP[@]}"
E

ENTRYPOINT ["/run.sh"]
