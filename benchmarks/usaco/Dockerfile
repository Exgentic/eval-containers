# syntax=docker/dockerfile:1
# USACO (USA Computing Olympiad)
# Shared environment — 304 tasks, competitive programming.
# Agent writes answer to /app/answer.txt.

FROM python:3.13-slim-bookworm

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="usaco"
LABEL dock.benchmark.description="USACO - competitive programming"
LABEL dock.benchmark.tasks="304"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y gcc g++ && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ENV BENCHMARK=usaco

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
