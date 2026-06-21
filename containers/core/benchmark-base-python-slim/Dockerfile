# syntax=docker/dockerfile:1
# Thin python runtime for python-GRADED HF benchmarks: benchmark-base-hf without
# uv/pyarrow/sitecustomize (~163MB). The dataset is materialized in a discarded
# duckdb builder and COPYd into /tasks, so the runtime keeps only what graders
# need: python + jq + /eval-grade + /eval-materialize-task. Unlike benchmark-base-slim
# it sets no ENTRYPOINT/CMD — each python-graded benchmark supplies its own grader.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM python:3.12-slim

LABEL eval.type="core-base"
LABEL eval.base.kind="python-slim"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           jq ca-certificates; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
COPY --from=entrypoint /eval-grade /eval-grade
RUN chmod +x /eval-materialize-task /eval-grade && mkdir -p /tasks /tests /app
WORKDIR /app
