# USACO (USA Computing Olympiad)
# Shared environment — 304 tasks, competitive programming.
# Agent writes answer to /app/answer.txt.

FROM python:3.13-slim-bookworm

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="usaco"
LABEL dock.benchmark.description="USACO - competitive programming"
LABEL dock.benchmark.tasks="304"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y gcc g++ && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ENV BENCHMARK=usaco

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
