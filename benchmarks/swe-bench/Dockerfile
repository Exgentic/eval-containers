# syntax=docker/dockerfile:1
# SWE-bench Verified
# Per-task images from ghcr.io/epoch-research/swe-bench.eval.x86_64.<task_id>:latest
# Upstream image includes repo state, tests, and evaluation harness.

ARG TASK_ID
FROM ghcr.io/epoch-research/swe-bench.eval.x86_64.${TASK_ID}:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="swebench"
LABEL dock.benchmark.description="SWE-bench Verified - software engineering"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"

ENV BENCHMARK=swebench-verified
ENV TASK_ID=${TASK_ID}

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
