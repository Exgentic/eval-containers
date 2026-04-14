# CompileBench (QuesmaOrg)
# 15 per-task images — agent compiles open source projects.
# Upstream: https://github.com/QuesmaOrg/CompileBench (compilebench branch)
# Tasks are in Harbor format — each has environment/Dockerfile, instruction.md, tests/test.sh.
#
# This Dockerfile fetches a single task's assets from the upstream repo
# and sets up the Dock entrypoint. The base image comes from the task's own Dockerfile.
#
# Build: docker build --build-arg TASK_ID=curl -t quay.io/dock-eval/benchmarks/compilebench-curl:latest .

ARG TASK_ID
ARG BASE_IMAGE=ubuntu:22.04

FROM ${BASE_IMAGE}

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="compilebench"
LABEL dock.benchmark.description="CompileBench - compile open source projects"
LABEL dock.benchmark.tasks="15"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="66e27468505706643088b79f8efad6260c274dc5"
LABEL dock.benchmark.url="https://github.com/QuesmaOrg/CompileBench"

ARG TASK_ID
ENV BENCHMARK=compilebench
ENV TASK_ID=${TASK_ID}

RUN apt-get update && apt-get install -y curl 2>/dev/null || apk add --no-cache curl

# Fetch task instruction and tests from upstream
ARG COMPILEBENCH_REV=66e27468505706643088b79f8efad6260c274dc5
RUN mkdir -p /tasks/0 /tests && \
    curl -sfL "https://raw.githubusercontent.com/QuesmaOrg/CompileBench/${COMPILEBENCH_REV}/datasets/compilebench/${TASK_ID}/instruction.md" \
      -o /tasks/0/problem.txt && \
    echo "${TASK_ID}" > /tasks/0/id.txt && \
    curl -sfL "https://raw.githubusercontent.com/QuesmaOrg/CompileBench/${COMPILEBENCH_REV}/datasets/compilebench/${TASK_ID}/tests/test.sh" \
      -o /tests/test.sh && \
    chmod +x /tests/test.sh
RUN chmod -R 600 /tasks

WORKDIR /workdir

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/0/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
