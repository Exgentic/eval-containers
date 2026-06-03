# syntax=docker/dockerfile:1
# Shared base for HuggingFace-dataset-backed benchmarks (Rule 11b).
# Provides:
#   - apt-retry wrapper (network flake defence)
#   - uv-installed pyarrow (10x faster + better connection reuse than pip)
#   - sitecustomize.py that monkey-patches urllib.request with retries,
#     so every subclass's `RUN python3 <<'PYEOF' urllib.request.urlretrieve(...)`
#     auto-retries on transient HF failures (one fix, rule 11 applied).
#   - /eval-entrypoint.sh + /eval-materialize-task copied from core/entrypoint
#   - /tasks /tests /app scaffolding
# Pin amd64 so runtime arch matches what agents' COPYd binaries expect
# (see core/agent-base-node Dockerfile for the full rationale).
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM --platform=linux/amd64 python:3.12-slim
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="core-base"
LABEL eval.base.kind="hf"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates git jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

# uv: faster package manager with better retry semantics than stock pip.
# Install via the official standalone installer into /usr/local/bin so both
# `uv` and `uv pip` are on PATH without touching system Python.
# uv via PyPI (more reliable than astral.sh's redirect chain to github).
# Pin version for reproducibility. Retry loop for transient PyPI flake.
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if pip install --no-cache-dir --retries 5 --timeout 60 uv==0.5.14; then ok=1; break; fi; \
      echo "pip install uv attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "pip install uv failed after 5 tries" >&2; exit 1; }

# pyarrow via uv. `uv pip install --system` writes to the image's system
# python (no virtualenv) — matches what benchmarks expect when they
# `import pyarrow` in a heredoc.
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

# sitecustomize.py is picked up automatically by the `site` module at every
# `python3` startup if it lives on sys.path. site-packages is the canonical
# location. Tests this by running `python3 -c 'import sys; print(sys.modules["sitecustomize"])'`.
COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest /eval-entrypoint.sh /eval-entrypoint.sh
COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest /eval-materialize-task /eval-materialize-task
COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest /eval-sitecustomize.py /usr/local/lib/python3.12/site-packages/sitecustomize.py
RUN chmod +x /eval-entrypoint.sh /eval-materialize-task

RUN mkdir -p /tasks /tests /app
WORKDIR /app

# HF token plumbing — single home (rule 11) for the 4 HF-gated child
# Dockerfiles (gaia, flores200, hle, frontiermath). CI passes the token
# via `--mount=type=secret,id=HF_TOKEN` (preferred, production path).
# The local test harness lacks `testcontainers` secret support, so it
# also accepts `--build-arg HF_TOKEN=...` — the ARG→ENV bridge here
# lets every child image's `echo "$HF_TOKEN"` fallback in their secret-
# reader see it without per-child ARG declarations.
ARG HF_TOKEN=""
ENV HF_TOKEN=$HF_TOKEN
