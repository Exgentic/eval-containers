# syntax=docker/dockerfile:1
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

# Test: compare /app/answer.txt to EXPECTED_ANSWER env var
RUN mkdir -p /tests
RUN <<'EOF'
cat > /tests/test.sh <<'TEST'
#!/bin/bash
set -euo pipefail
mkdir -p /logs/verifier
ACTUAL=$(cat /app/answer.txt 2>/dev/null | tr -d "[:space:]")
if [ "$ACTUAL" = "$EXPECTED_ANSWER" ]; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
chmod +x /tests/test.sh
EOF

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
