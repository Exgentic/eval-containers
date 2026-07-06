# syntax=docker/dockerfile:1
# Shared base for GitHub-raw-file-backed benchmarks (Rule 11b).
# Provides: apt-retry wrapper, curl + jq, entrypoint + materialize copy, /tasks /tests /app.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM python:3.12-slim

LABEL eval.type="core-base"
LABEL eval.base.kind="github"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing \
         && apt-get upgrade -y \
         && apt-get install -y --no-install-recommends \
           curl ca-certificates git jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

RUN mkdir -p /tasks /tests /app
WORKDIR /app
