# CompileBench (QuesmaOrg)
# 15 per-task images — agent compiles open source projects.
# Upstream: https://github.com/QuesmaOrg/CompileBench (compilebench branch)
# Tasks are in Harbor format — each has environment/Dockerfile, instruction.md, tests/test.sh.
#
# This Dockerfile fetches a single task's assets from the upstream repo
# and sets up the Eval Containers entrypoint. The base image comes from the task's own Dockerfile.
#
# Build: docker build --build-arg EVAL_TASK_ID=curl -t quay.io/eval-containers/benchmarks/compilebench-curl:latest .

ARG EVAL_TASK_ID
ARG BASE_IMAGE=ubuntu:22.04

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${BASE_IMAGE}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="compilebench"
LABEL eval.benchmark.description="CompileBench - compile open source projects"
LABEL eval.benchmark.tasks="15"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="66e27468505706643088b79f8efad6260c274dc5"
LABEL eval.benchmark.url="https://github.com/QuesmaOrg/CompileBench"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="66e27468505706643088b79f8efad6260c274dc5"

ARG EVAL_TASK_ID
ENV EVAL_BENCHMARK=compilebench
ENV EVAL_COMPILEBENCH_UPSTREAM_ID=${EVAL_TASK_ID}
ENV EVAL_TASK_ID=0

# Install curl + python3 on whatever base the task's Dockerfile picks.
# Retry apt-get up to 3 times to survive transient network hiccups, then
# fall back to apk (alpine) if apt is not present at all.
RUN set -eux; \
    if command -v apt-get >/dev/null 2>&1; then \
      success=0; \
      for i in 1 2 3 4 5; do \
        if apt-get update && apt-get install -y --no-install-recommends ca-certificates curl python3; then \
          success=1; break; \
        fi; \
        echo "apt attempt $i failed; retrying" >&2; sleep $((i*5)); \
      done; \
      [ "$success" = "1" ] || { echo "apt-get failed after 5 attempts" >&2; exit 1; }; \
      rm -rf /var/lib/apt/lists/*; \
    elif command -v apk >/dev/null 2>&1; then \
      for i in 1 2 3; do apk add --no-cache curl python3 && break || sleep $((i*5)); done; \
    else \
      echo "no supported package manager on base image" >&2; exit 1; \
    fi; \
    command -v curl >/dev/null 2>&1 || { echo "curl still missing after install" >&2; exit 1; }

# Fetch task instruction and tests from upstream, then write a single-row
# /tasks/all.jsonl. Single-JSONL task layout (see benchmarks/RULES.md #22).
ARG COMPILEBENCH_REV=66e27468505706643088b79f8efad6260c274dc5
RUN mkdir -p /tasks /tests && \
    BASE="https://raw.githubusercontent.com/QuesmaOrg/CompileBench/${COMPILEBENCH_REV}/datasets/compilebench/${EVAL_COMPILEBENCH_UPSTREAM_ID}" && \
    curl -sfL --retry 5 --retry-delay 3 --retry-connrefused --max-time 120 \
      "$BASE/instruction.md" -o /tmp/instruction.md && \
    curl -sfL --retry 5 --retry-delay 3 --retry-connrefused --max-time 120 \
      "$BASE/tests/test.sh" -o /grade.sh && \
    chmod +x /grade.sh && \
    python3 -c "import json; open('/tasks/all.jsonl','w').write(json.dumps({'id':'${EVAL_COMPILEBENCH_UPSTREAM_ID}','problem':open('/tmp/instruction.md').read()})+'\n')" && \
    rm -f /tmp/instruction.md
RUN chmod 600 /tasks/all.jsonl

WORKDIR /workdir

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
