# Slim runtime base for shell-graded HF benchmarks: debian-slim + jq, no python.
# Dataset materialized in a discarded duckdb builder and COPYd into /tasks here.
# Bundles the shared entrypoint + exact-match grader, so a benchmark supplies
# only its labels, loader, and EVAL_TASK_PROMPT.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
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
COPY eval-entrypoint /entrypoint.sh
COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /eval-materialize-task /entrypoint.sh /grade.sh && mkdir -p /tasks /tests /app
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
