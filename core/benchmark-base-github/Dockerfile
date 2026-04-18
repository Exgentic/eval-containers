# syntax=docker/dockerfile:1
# Shared base for GitHub-raw-file-backed benchmarks (Rule 11b).
# Provides: apt-retry wrapper, curl + jq, entrypoint + materialize copy, /tasks /tests /app.
FROM --platform=linux/amd64 python:3.12-slim

LABEL dock.type="core-base"
LABEL dock.base.kind="github"

RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           curl ca-certificates git jq; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }; \
    rm -rf /var/lib/apt/lists/*

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN mkdir -p /tasks /tests /app
WORKDIR /app
