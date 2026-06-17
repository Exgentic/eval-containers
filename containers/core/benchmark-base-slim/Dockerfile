# Slim runtime base for shell-graded HF benchmarks: debian-slim + jq, no python.
# Dataset materialized in a discarded duckdb builder and COPYd into /tasks here.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM --platform=linux/amd64 debian:12-slim

LABEL eval.type="core-base"
LABEL eval.base.kind="slim"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           jq ca-certificates; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

COPY eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task && mkdir -p /tasks /tests /app
WORKDIR /app
