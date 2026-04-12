# syntax=docker/dockerfile:1
# AIME (American Invitational Mathematics Examination)
# Shared environment — 60 tasks, integer answer 1-999.

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="60"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ENV BENCHMARK=aime

# Generic test: compare /app/answer.txt to EXPECTED_ANSWER
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
