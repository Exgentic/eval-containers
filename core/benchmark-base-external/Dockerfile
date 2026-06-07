# syntax=docker/dockerfile:1
# Shared base for externally-graded benchmarks (Rule 11b, compose/RULES.md #16).
# Same network-resilience stack as core/benchmark-base-hf:
#   - apt-retry wrapper
#   - uv for pyarrow
#   - sitecustomize.py — urllib retries for subclass heredocs
# Plus: default /tests/test.sh writes reward=-1 so externally-graded
# benchmarks start from a known-bad baseline that their real grader
# overrides.
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM --platform=linux/amd64 python:3.12-slim

LABEL eval.type="core-base"
LABEL eval.base.kind="external"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates git jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

# uv via PyPI (more reliable than astral.sh's redirect chain to github).
# Pin version for reproducibility. Retry loop for transient PyPI flake.
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries 5 --timeout 60 uv==0.5.14; then ok=1; break; fi; \
      echo "pip install uv attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "pip install uv failed after 5 tries" >&2; exit 1; }

# uv pip retry loop — PyPI/pythonhosted metadata fetches can stall on
# flaky podman networks. `UV_HTTP_TIMEOUT=120` lifts uv's default 30s,
# plus a 5-shot outer retry with exponential backoff.
ENV UV_HTTP_TIMEOUT=120
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if uv pip install --system --no-cache pyarrow==18.1.0; then ok=1; break; fi; \
      echo "uv pip attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "uv pip install failed after 5 tries" >&2; exit 1; }

COPY --from=entrypoint /eval-entrypoint.sh /eval-entrypoint.sh
COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
COPY --from=entrypoint /eval-sitecustomize.py /usr/local/lib/python3.12/site-packages/sitecustomize.py
RUN chmod +x /eval-entrypoint.sh /eval-materialize-task

RUN mkdir -p /tasks /tests /app && \
    printf '#!/bin/bash\nmkdir -p /logs/verifier\necho -1 > /logs/verifier/reward.txt\n' > /grade.sh && \
    chmod +x /grade.sh
WORKDIR /app
