# Kumo (data science benchmark)
# Shared environment — 250 hard tasks.
# Agent writes answer to /app/answer.txt.

FROM python:3.11-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="kumo"
LABEL dock.benchmark.description="Kumo - data science benchmark"
LABEL dock.benchmark.tasks="250"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN pip install --no-cache-dir requests==2.31.0

WORKDIR /app
ENV BENCHMARK=kumo

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
