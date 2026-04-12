# syntax=docker/dockerfile:1
# BFCL (Berkeley Function Calling Leaderboard)
# Shared environment — 3641 tasks, function/tool calling evaluation.
# Agent writes answer to /app/answer.txt.

FROM python:3.10-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="bfcl"
LABEL dock.benchmark.description="Berkeley Function Calling Leaderboard"
LABEL dock.benchmark.tasks="3641"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN pip install --no-cache-dir pytest

WORKDIR /app
ENV BENCHMARK=bfcl
ENV PYTHONUNBUFFERED=1

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
