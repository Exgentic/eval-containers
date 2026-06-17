# Builder base for HF-dataset benchmarks: duckdb, no python. Benchmarks
# `FROM ... AS data`, run /eval-hf-load, then a slim stage COPYs /tasks out.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM debian:12-slim

LABEL eval.type="core-base"
LABEL eval.base.kind="duckdb-builder"

# ≥1.5: 1.1.x mis-resolves the @~parquet auto-convert alias.
ARG DUCKDB_VERSION=1.5.3
ARG TARGETARCH

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates unzip jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if curl -fsSL "https://github.com/duckdb/duckdb/releases/download/v${DUCKDB_VERSION}/duckdb_cli-linux-${TARGETARCH}.zip" -o /tmp/d.zip; then ok=1; break; fi; \
      echo "duckdb download attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "duckdb download failed after 5 tries" >&2; exit 1; }; \
    unzip -q /tmp/d.zip -d /usr/local/bin && rm /tmp/d.zip && \
    duckdb -c "INSTALL httpfs;" && duckdb --version

COPY eval-hf-load /eval-hf-load
RUN chmod +x /eval-hf-load && mkdir -p /tasks
WORKDIR /app
