# syntax=docker/dockerfile:1
# GPQA Diamond
# Shared environment — 198 tasks, graduate-level science questions.
# Agent writes answer to /app/answer.txt.

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gpqa-diamond"
LABEL dock.benchmark.description="GPQA Diamond - graduate-level science"
LABEL dock.benchmark.tasks="198"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

WORKDIR /app
ENV BENCHMARK=gpqa-diamond

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
