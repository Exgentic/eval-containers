# syntax=docker/dockerfile:1
# Trimmed litellm proxy image. Used as the base for litellm-flavored
# model images (models/<name>/Dockerfile FROMs this and adds config).
#
# Layout (uniform across gateway flavors so combination.Dockerfile can
# COPY a single tree):
#   /opt/gateway/start         executable that launches the gateway
#   /opt/gateway/config.yaml   provided by the model image (litellm yaml)
#   /opt/venv/                 the python venv with litellm
#
# Honest size: ~525 MB. litellm[proxy] hard-imports many provider SDKs
# (boto3, google-cloud, anthropic, ...) at startup so we can only
# safely strip prisma. The baseline cost of a Python LLM proxy.
FROM --platform=linux/amd64 python:3.12-slim AS builder

COPY --from=ghcr.io/astral-sh/uv:0.5.14 /uv /usr/local/bin/uv

# venv lives at /opt/gateway/venv so the entire gateway tree is
# self-contained under /opt/gateway/ — combination.Dockerfile only has
# to COPY one path and gets the binary, the venv, and the config.
ENV UV_HTTP_TIMEOUT=120 UV_NO_CACHE=1
RUN uv venv --python 3.12 --seed --relocatable /opt/gateway/venv \
 && uv pip install --python /opt/gateway/venv/bin/python \
      "litellm[proxy]==1.83.3" \
      "opentelemetry-api>=1.25.0" \
      "opentelemetry-sdk>=1.25.0" \
      "opentelemetry-exporter-otlp-proto-http>=1.25.0"

# Drop prisma — DB ORM only used when virtual keys / DB-backed budgets
# are enabled. Other heavy deps (boto3, google, anthropic) are imported
# unconditionally at proxy startup so cannot be stripped without a
# fork. Strip pyc caches and test dirs to claw back ~10 MB.
RUN /opt/gateway/venv/bin/pip uninstall -y prisma || true \
 && find /opt/gateway/venv -type d -name '__pycache__' -exec rm -rf {} + 2>/dev/null || true \
 && find /opt/gateway/venv -type f -name '*.pyc' -delete 2>/dev/null || true \
 && find /opt/gateway/venv -type d \( -name 'tests' -o -name 'test' \) -exec rm -rf {} + 2>/dev/null || true

FROM --platform=linux/amd64 python:3.12-slim

LABEL gateway.kind="litellm"
LABEL gateway.litellm_version="1.83.3"
LABEL gateway.translates_protocols="true"

# curl is the uniform health-check tool across gateway flavors. The
# eval image (FROM benchmark-base-hf, debian) already has it; we add
# it here so the same /opt/gateway/health works when this image runs
# standalone in compose mode.
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates gettext-base \
 && rm -rf /var/lib/apt/lists/*

# Caddy: tiny shim that maps the framework's protocol-namespaced URLs
# (/anthropic/*, /openai/*, /<protocol>/*) to litellm's native root
# paths (/v1/messages, /v1/chat/completions, /v1/responses). Same
# pattern for any non-bifrost gateway; the prefixes are extensible.
COPY --from=docker.io/library/caddy:2.8-alpine /usr/bin/caddy /usr/bin/caddy

COPY --from=builder /opt/gateway/venv /opt/gateway/venv
COPY start    /opt/gateway/start
COPY health   /opt/gateway/health
COPY Caddyfile /opt/gateway/Caddyfile
RUN chmod +x /opt/gateway/start /opt/gateway/health

ENV PYTHONDONTWRITEBYTECODE=1

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
