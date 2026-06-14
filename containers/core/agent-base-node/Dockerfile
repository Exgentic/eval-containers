# syntax=docker/dockerfile:1
# Shared base for node-runtime agents (Rule 11a).
# Provides: apt-retry wrapper, node tarball install with arch detect, /opt/agent skeleton.
# Pin platform: benchmark bases are amd64 (python:3.12-slim pulls amd64
# by default on our build hosts), so the combo runtime is amd64. Node
# binaries COPYd from here into the combo must match that arch or fail
# with "cannot execute: required file not found" under Rosetta. CI on
# Linux amd64 is a no-op; on Apple Silicon, this forces Rosetta-assisted
# amd64 builds so the produced node binary matches what runs at runtime.
FROM --platform=linux/amd64 ubuntu:24.04
LABEL eval.type="core-base"
LABEL eval.base.runtime="node"
# Bumped 22.11.0 → 22.20.0 (latest LTS Node 22 release):
#   - openclaw explicitly requires Node ≥22.14
#   - newer node flags (e.g. --disable-sigusr1) used by some agent
#     CLIs require >= 22.14
#   - stays on Node 22 LTS so we don't break any agent expecting LTS
#     behaviour (Node 23 is current but not LTS)
ARG NODE_VERSION=22.20.0
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates xz-utils git; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*
RUN set -e; for i in 1 2 3; do \
      ARCH=$(dpkg --print-architecture 2>/dev/null || uname -m); \
      case "$ARCH" in arm64|aarch64) NA=linux-arm64 ;; *) NA=linux-x64 ;; esac; \
      curl -fsSL --retry 3 --retry-delay 5 --connect-timeout 30 \
        "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-${NA}.tar.xz" -o /tmp/n.tar.xz && \
      [ "$(stat -c %s /tmp/n.tar.xz)" -gt 10000000 ] && break || \
      (echo "node dl attempt $i failed; retry" >&2; sleep 5); \
    done && mkdir -p /opt/agent/node && \
    tar -xJf /tmp/n.tar.xz -C /opt/agent/node --strip-components=1 && rm /tmp/n.tar.xz && \
    ln -sf /opt/agent/node/bin/node /usr/local/bin/node && \
    ln -sf /opt/agent/node/bin/npm  /usr/local/bin/npm && \
    ln -sf /opt/agent/node/bin/npx  /usr/local/bin/npx
ENV PATH=/opt/agent/node/bin:$PATH
# Network-flake defence for `npm install -g ...` in subclasses. Registry
# metadata + tarball fetches under concurrent builds frequently 5xx or
# reset; stock npm gives up after 2 retries. One config in the base
# lifts every subclass (rule 11).
RUN npm config set fetch-retries 10 --global && \
    npm config set fetch-retry-mintimeout 20000 --global && \
    npm config set fetch-retry-maxtimeout 120000 --global && \
    npm config set fetch-timeout 600000 --global
RUN printf '#!/bin/bash\nset -euo pipefail\n# Default skeleton: subclass agent Dockerfiles overwrite this.\nfor b in node npm npx; do ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b 2>/dev/null || true; done\n' > /opt/agent/install.sh && chmod +x /opt/agent/install.sh && \
    printf '#!/bin/bash\necho "ERROR: agent-base entrypoint not overridden by subclass" >&2\nexit 1\n' > /run.sh && chmod +x /run.sh
