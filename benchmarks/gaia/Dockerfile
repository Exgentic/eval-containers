# GAIA (General AI Assistants)
# Shared environment — 165 tasks. Requires internet access.
# Agent writes answer to /app/answer.txt.

FROM python:3.11-slim-bookworm

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gaia"
LABEL dock.benchmark.description="GAIA - General AI Assistants"
LABEL dock.benchmark.tasks="165"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"

RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ENV BENCHMARK=gaia

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
