# syntax=docker/dockerfile:1
# Plandex agent image — self-hosted client + server with Postgres + internal litellm.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
ARG AGENT_VERSION=2.2.1
FROM plandexai/plandex-server:server-v${AGENT_VERSION} AS plandex-server-src
# Source of the bundled uv binary (see install.sh).
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest AS uv-src
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-rust:latest
ARG AGENT_VERSION=2.2.1
LABEL eval.type="agent"
LABEL eval.agent.name="plandex"
LABEL eval.agent.description="Plandex terminal AI coding agent (self-hosted)"
LABEL eval.agent.runtime="go"
LABEL eval.agent.url="https://github.com/plandex-ai/plandex"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=plandex
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        postgresql-16 postgresql-client-16 python3 python3-venv libpq5 && rm -rf /var/lib/apt/lists/*
# Stage everything plandex needs at runtime under /opt/agent/ so the
# combination.Dockerfile's `COPY --from=AGENT_IMAGE /opt/agent/` picks
# it all up — files outside /opt/agent/ don't reach the eval image.
COPY --from=plandex-server-src /app/server/plandex-server /opt/agent/bin/plandex-server
COPY --from=plandex-server-src /app/server/migrations /opt/agent/plandex/migrations
COPY --from=plandex-server-src /app/server/litellm_proxy.py /opt/agent/plandex/litellm_proxy.py
COPY --from=plandex-server-src /scripts /opt/agent/plandex/scripts
COPY --from=plandex-server-src /opt/venv /opt/agent/venv
COPY --from=uv-src /opt/agent/uv /opt/agent/uv
RUN ARCH=$(uname -m) && case "$ARCH" in x86_64) A=amd64 ;; aarch64|arm64) A=arm64 ;; *) exit 1 ;; esac && \
    curl -fsSL "https://github.com/plandex-ai/plandex/releases/download/cli/v${AGENT_VERSION}/plandex_${AGENT_VERSION}_linux_${A}.tar.gz" -o /tmp/p.tgz && \
    tar -xzf /tmp/p.tgz -C /opt/agent/bin plandex && chmod +x /opt/agent/bin/plandex && \
    ln -sf /opt/agent/bin/plandex /usr/local/bin/plandex && rm /tmp/p.tgz
COPY entrypoint.sh /run.sh
COPY custom-models.json /opt/agent/custom-models.json
RUN chmod +x /run.sh && \
    cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
# Install postgres-16 from the upstream pgdg repo (debian ships 15) + rebuild
# plandex's litellm venv. Versions match plandex's own scripts/litellm_deps.sh.
apt-get update --fix-missing
apt-get install -y --no-install-recommends curl ca-certificates gnupg lsb-release
install -d /usr/share/postgresql-common/pgdg
curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc \
  -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc
. /etc/os-release
echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt ${VERSION_CODENAME}-pgdg main" \
  > /etc/apt/sources.list.d/pgdg.list
apt-get update
apt-get install -y --no-install-recommends postgresql-16 postgresql-client-16 libpq5
rm -rf /var/lib/apt/lists/*
# uv supplies its own Python — the benchmark base may have none. Land it in
# /opt (agent-readable; the combo image's /root is 0700).
export UV_PYTHON_INSTALL_DIR=/opt/uv-python
/opt/agent/uv python install 3.12
rm -rf /opt/agent/venv
/opt/agent/uv venv --python 3.12 /opt/agent/venv
/opt/agent/uv pip install --no-cache --python /opt/agent/venv/bin/python \
  litellm==1.72.6 fastapi==0.115.12 uvicorn==0.34.1 \
  google-cloud-aiplatform==1.96.0 boto3==1.38.40 botocore==1.38.40
chmod -R a+rX /opt/uv-python /opt/agent/venv
[ -x /opt/agent/bin/plandex ] && ln -sf /opt/agent/bin/plandex /usr/local/bin/plandex || true
[ -x /opt/agent/bin/plandex-server ] && ln -sf /opt/agent/bin/plandex-server /usr/local/bin/plandex-server || true
I

ENTRYPOINT ["/run.sh"]
