# syntax=docker/dockerfile:1
# Plandex agent image — self-hosted client+server with Postgres + internal litellm.
FROM plandexai/plandex-server:server-v2.2.1 AS plandex-server-src
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
LABEL eval.type="agent"
LABEL eval.agent.name="plandex"
LABEL eval.agent.description="Plandex terminal AI coding agent (self-hosted)"
LABEL eval.agent.runtime="go"
LABEL eval.agent.url="https://github.com/plandex-ai/plandex"
LABEL eval.agent.version="2.2.1"
ENV EVAL_AGENT_VERSION_DEFAULT="2.2.1"
ENV EVAL_AGENT=plandex
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        postgresql-16 postgresql-client-16 python3 python3-venv libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=plandex-server-src /app/server/plandex-server /usr/local/bin/plandex-server
COPY --from=plandex-server-src /app/server/migrations /opt/plandex/migrations
COPY --from=plandex-server-src /scripts /opt/plandex/scripts
COPY --from=plandex-server-src /opt/venv /opt/venv
RUN ARCH=$(uname -m) && case "$ARCH" in x86_64) A=amd64 ;; aarch64|arm64) A=arm64 ;; *) exit 1 ;; esac && \
    curl -fsSL "https://github.com/plandex-ai/plandex/releases/download/cli/v2.2.1/plandex_2.2.1_linux_${A}.tar.gz" -o /tmp/p.tgz && \
    tar -xzf /tmp/p.tgz -C /opt/agent/bin plandex && chmod +x /opt/agent/bin/plandex && \
    ln -sf /opt/agent/bin/plandex /usr/local/bin/plandex && rm /tmp/p.tgz
COPY entrypoint.sh /opt/agent/entrypoint.sh
RUN chmod +x /opt/agent/entrypoint.sh && \
    printf '#!/bin/bash\nset -euo pipefail\n[ -x /opt/agent/bin/plandex ] && ln -sf /opt/agent/bin/plandex /usr/local/bin/plandex || true\n' > /opt/agent/install.sh && chmod +x /opt/agent/install.sh
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
