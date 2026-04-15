# CompileBench (QuesmaOrg)
# 15 per-task images — agent compiles open source projects.
# Upstream: https://github.com/QuesmaOrg/CompileBench (compilebench branch)
# Tasks are in Harbor format — each has environment/Dockerfile, instruction.md, tests/test.sh.
#
# This Dockerfile fetches a single task's assets from the upstream repo
# and sets up the Dock entrypoint. The base image comes from the task's own Dockerfile.
#
# Build: docker build --build-arg DOCK_TASK_ID=curl -t quay.io/dock-eval/benchmarks/compilebench-curl:latest .

ARG DOCK_TASK_ID
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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="66e27468505706643088b79f8efad6260c274dc5"

ARG DOCK_TASK_ID
ENV DOCK_BENCHMARK=compilebench
ENV DOCK_COMPILEBENCH_UPSTREAM_ID=${DOCK_TASK_ID}
ENV DOCK_TASK_ID=0

RUN (apt-get update && apt-get install -y curl python3 && rm -rf /var/lib/apt/lists/*) 2>/dev/null || apk add --no-cache curl python3

# Fetch task instruction and tests from upstream, then write a single-row
# /tasks/all.jsonl. Single-JSONL task layout (see benchmarks/RULES.md #22).
ARG COMPILEBENCH_REV=66e27468505706643088b79f8efad6260c274dc5
RUN mkdir -p /tasks /tests && \
    curl -sfL "https://raw.githubusercontent.com/QuesmaOrg/CompileBench/${COMPILEBENCH_REV}/datasets/compilebench/${DOCK_COMPILEBENCH_UPSTREAM_ID}/instruction.md" \
      -o /tmp/instruction.md && \
    curl -sfL "https://raw.githubusercontent.com/QuesmaOrg/CompileBench/${COMPILEBENCH_REV}/datasets/compilebench/${DOCK_COMPILEBENCH_UPSTREAM_ID}/tests/test.sh" \
      -o /tests/test.sh && \
    chmod +x /tests/test.sh && \
    python3 -c "import json; open('/tasks/all.jsonl','w').write(json.dumps({'id':'${DOCK_COMPILEBENCH_UPSTREAM_ID}','problem':open('/tmp/instruction.md').read()})+'\n')" && \
    rm -f /tmp/instruction.md
RUN chmod 600 /tasks/all.jsonl

WORKDIR /workdir

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
