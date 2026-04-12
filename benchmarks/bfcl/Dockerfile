# BFCL (Berkeley Function Calling Leaderboard)
# Shared environment — 3641 tasks, function/tool calling evaluation.
# Agent writes answer to /app/answer.txt.

FROM python:3.10-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="bfcl"
LABEL dock.benchmark.description="Berkeley Function Calling Leaderboard"
LABEL dock.benchmark.tasks="3641"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN pip install --no-cache-dir pytest

WORKDIR /app
ENV BENCHMARK=bfcl
ENV PYTHONUNBUFFERED=1

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
