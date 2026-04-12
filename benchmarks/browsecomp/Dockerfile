# BrowseComp (OpenAI)
# Shared environment — 1266 tasks. Agent must browse the live web.
# Agent writes answer to /app/answer.txt.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="browsecomp"
LABEL dock.benchmark.description="BrowseComp - web browsing QA"
LABEL dock.benchmark.tasks="1266"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"

RUN apt-get update && apt-get install -y \
    curl wget chromium chromium-driver \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir playwright requests && playwright install

WORKDIR /app
ENV BENCHMARK=browsecomp

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
