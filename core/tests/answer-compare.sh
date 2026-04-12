#!/bin/bash
# Generic test: compare /app/answer.txt to EXPECTED_ANSWER env var.
# Used by shared-env benchmarks where the agent writes a single answer.
set -euo pipefail
mkdir -p /logs/verifier
ACTUAL=$(cat /app/answer.txt 2>/dev/null | tr -d "[:space:]")
if [ "$ACTUAL" = "$EXPECTED_ANSWER" ]; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
