# syntax=docker/dockerfile:1
# Crush agent image (Charm)
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
LABEL eval.type="agent"
LABEL eval.agent.name="crush"
LABEL eval.agent.description="Charm Crush terminal AI coding assistant"
LABEL eval.agent.runtime="go"
LABEL eval.agent.url="https://github.com/charmbracelet/crush"
LABEL eval.agent.version="0.57.0"
ENV EVAL_AGENT_VERSION_DEFAULT="0.57.0"
ENV EVAL_AGENT=crush
# Download-to-file with retry + size check; avoids streaming curl|tar
# corruption on partial bytes under flaky networks (same pattern as
# agents/goose/Dockerfile).
RUN set -e; ok=0; ARCH=$(uname -m); \
    case "$ARCH" in x86_64) CA=x86_64 ;; aarch64|arm64) CA=arm64 ;; *) echo "unsupported arch $ARCH" >&2; exit 1 ;; esac; \
    URL="https://github.com/charmbracelet/crush/releases/download/v0.57.0/crush_0.57.0_Linux_${CA}.tar.gz"; \
    for i in 1 2 3 4 5; do \
      if curl -fsSL --retry 5 --retry-delay 3 --retry-connrefused --max-time 300 \
           "$URL" -o /tmp/crush.tar.gz \
         && [ "$(stat -c %s /tmp/crush.tar.gz)" -gt 1000000 ]; then \
        ok=1; break; \
      fi; \
      echo "crush tarball attempt $i failed; retry" >&2; rm -f /tmp/crush.tar.gz; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "crush tarball download failed" >&2; exit 1; }; \
    tar xz -C /tmp -f /tmp/crush.tar.gz && rm /tmp/crush.tar.gz && \
    install -m 0755 /tmp/crush_0.57.0_Linux_${CA}/crush /opt/agent/bin/crush && \
    ln -sf /opt/agent/bin/crush /usr/local/bin/crush && rm -rf /tmp/crush_0.57.0_Linux_${CA}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
[ -x /opt/agent/bin/crush ] && ln -sf /opt/agent/bin/crush /usr/local/bin/crush || true
I

COPY entrypoint.sh /run.sh
RUN chmod +x /run.sh
ENTRYPOINT ["/run.sh"]
