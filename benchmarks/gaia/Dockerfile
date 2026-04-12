# syntax=docker/dockerfile:1
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
