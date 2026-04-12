# SimpleQA (OpenAI)
# Shared environment — 4326 tasks, simple factual Q&A.
# Agent writes answer to /app/answer.txt.

FROM python:3.11-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="simpleqa"
LABEL dock.benchmark.description="SimpleQA - factual question answering"
LABEL dock.benchmark.tasks="4326"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
ENV BENCHMARK=simpleqa

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
