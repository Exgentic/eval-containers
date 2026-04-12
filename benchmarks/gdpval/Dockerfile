# syntax=docker/dockerfile:1
# GDPval
# 220 open tasks. Professional knowledge work.
# Scoring: LLM-as-judge via OpenAI grader.
# Needs internet for reference file download and grading API.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gdpval"
LABEL dock.benchmark.description="GDPval - professional knowledge work"
LABEL dock.benchmark.tasks="220"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"

RUN apt-get update && apt-get install -y \
    libreoffice curl wget \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir datasets requests

WORKDIR /app
ENV BENCHMARK=gdpval

# Test: GDPval uses LLM-as-judge, not simple answer comparison.
# TODO: integrate upstream grading API. For now, generic answer check.
RUN mkdir -p /tests
RUN <<'EOF'
cat > /tests/test.sh <<'TEST'
#!/bin/bash
set -euo pipefail
mkdir -p /logs/verifier
# GDPval scoring requires external LLM judge — placeholder.
# Agent produces deliverable files; grading happens externally.
if [ -f /app/deliverable.* ] || [ -f /app/answer.txt ]; then
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
