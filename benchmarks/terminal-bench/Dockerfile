# syntax=docker/dockerfile:1
# Terminal-Bench 2.0
# Built from upstream source at github.com/laude-institute/terminal-bench.
# Each task lives under original-tasks/<task>/ with its own Dockerfile, task.yaml,
# tests/, and run-tests.sh. We clone the repo at a pinned commit and layer the
# task on top of its upstream base image.

ARG EVAL_TASK_ID

# Stage 1: fetch the task directory from upstream at a pinned commit.
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM alpine/git:2.47.2 AS upstream
ARG EVAL_TASK_ID
ARG TBENCH_REF=1a6ffa9674b571da0ed040c470cb40c4d85f9b9b
RUN git clone https://github.com/laude-institute/terminal-bench.git /tb \
 && cd /tb && git checkout ${TBENCH_REF} \
 && test -d "/tb/original-tasks/${EVAL_TASK_ID}" \
 && cp -r "/tb/original-tasks/${EVAL_TASK_ID}" /task \
 && rm -rf /tb

# Stage 2: layer the task on top of the upstream shared base (a public image,
# anonymously pullable).
FROM ghcr.io/laude-institute/t-bench/python-3-13:20250620
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="terminal-bench"
LABEL eval.benchmark.upstream_base="ghcr.io/laude-institute/t-bench/python-3-13:20250620"
LABEL eval.benchmark.description="Terminal-Bench 2.0 - terminal tasks, built from upstream source"
LABEL eval.benchmark.tasks="100+"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/laude-institute/terminal-bench"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="2.0"

ARG EVAL_TASK_ID
ENV EVAL_BENCHMARK=tbench
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

RUN apt-get update \
 && apt-get install -y --no-install-recommends python3-yaml \
 && rm -rf /var/lib/apt/lists/*

# Copy the task tree from the upstream stage.
COPY --from=upstream /task /task

WORKDIR /app

# Extract task.yaml's instruction field into a plain-text file the entrypoint
# can read.
RUN python3 -c "import yaml,pathlib; y=yaml.safe_load(pathlib.Path('/task/task.yaml').read_text()); pathlib.Path('/task/instruction.md').write_text(y.get('instruction',''))"

# Move upstream tests + runner into a root-only /tests dir.
RUN mkdir -p /tests \
 && cp -r /task/tests /tests/tests \
 && cp /task/run-tests.sh /tests/run-tests.sh \
 && chmod +x /tests/run-tests.sh

# Scorer: run upstream run-tests.sh, translate pytest exit code into
# /logs/verifier/reward.txt (1 on pass, 0 on fail).
RUN cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
export TEST_DIR=/tests/tests
if /tests/run-tests.sh > /logs/verifier/test.log 2>&1; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
RUN chmod +x /tests/test.sh && chmod -R 700 /tests && chown -R root:root /tests

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest /eval-entrypoint.sh /eval-entrypoint.sh
RUN chmod +x /eval-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -z "$TASK" ] && [ -f /task/instruction.md ]; then
  export TASK="$(cat /task/instruction.md)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
