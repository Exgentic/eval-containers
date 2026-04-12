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

RUN mkdir -p /tests && printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ "$(cat /output/agent/stdout.log 2>/dev/null | tr -d "[:space:]")" = "$EXPECTED_ANSWER" ] && echo 1 > /logs/verifier/reward.txt || echo 0 > /logs/verifier/reward.txt\n' > /tests/test.sh && chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
