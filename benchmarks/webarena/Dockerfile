# WebArena Verified
# 812 tasks. Multi-service with website sidecars.
# Agent interacts with websites via Playwright.
# Task dataset downloaded at build time from upstream.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="webarena"
LABEL dock.benchmark.description="WebArena Verified - web browsing with sidecars"
LABEL dock.benchmark.tasks="812"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y \
    wget curl chromium chromium-driver \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir playwright && playwright install

WORKDIR /app
ENV BENCHMARK=webarena-verified

# Download task dataset from upstream at build time
RUN wget -q -O /app/tasks.json \
    https://raw.githubusercontent.com/ServiceNow/webarena-verified/main/assets/dataset/webarena-verified.json

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
