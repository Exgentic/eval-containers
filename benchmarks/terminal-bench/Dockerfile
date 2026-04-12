# syntax=docker/dockerfile:1
# Terminal-Bench 2.0
# Per-task images from ghcr.io/laude-institute/terminal-bench/<task>:2.0
# Upstream image includes tests and instruction.

ARG TASK_ID
FROM ghcr.io/laude-institute/terminal-bench/${TASK_ID}:2.0

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="terminal-bench"
LABEL dock.benchmark.description="Terminal-Bench 2.0 - terminal tasks"
LABEL dock.benchmark.tasks="89"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"

ENV BENCHMARK=terminal-bench
ENV TASK_ID=${TASK_ID}

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
