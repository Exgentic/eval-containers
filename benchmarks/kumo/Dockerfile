# syntax=docker/dockerfile:1
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
