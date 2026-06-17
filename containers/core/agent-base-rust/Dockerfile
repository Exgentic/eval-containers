# syntax=docker/dockerfile:1
# Shared base for agents that ship prebuilt binaries in /opt/agent/bin (Rule 11a).
# Label `eval.base.runtime="rust"` is the canonical tag; Go-based agents (like
# `crush`) also extend this base because the install shape is identical —
# drop a prebuilt binary + symlink into /usr/local/bin. If a Go-only base
# emerges, add a `core/agent-base-go/` and retag then.
# Provides: apt-retry wrapper with tar/bzip2, /opt/agent skeleton + /opt/agent/bin on PATH.
FROM ubuntu:24.04
LABEL eval.type="core-base"
LABEL eval.base.runtime="rust"
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing \
         && apt-get upgrade -y \
         && apt-get install -y --no-install-recommends \
           curl ca-certificates xz-utils bzip2 libgomp1 git; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*
RUN mkdir -p /opt/agent/bin && \
    printf '#!/bin/bash\nset -euo pipefail\n# Default skeleton: subclass agent Dockerfiles overwrite this.\nfor f in /opt/agent/bin/*; do [ -x "$f" ] && ln -sf "$f" /usr/local/bin/$(basename "$f") || true; done\n' > /opt/agent/install.sh && chmod +x /opt/agent/install.sh && \
    printf '#!/bin/bash\necho "ERROR: agent-base entrypoint not overridden by subclass" >&2\nexit 1\n' > /run.sh && chmod +x /run.sh
ENV PATH=/opt/agent/bin:$PATH
