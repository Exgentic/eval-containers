#!/bin/bash
# Exact-match test: compare agent stdout (whitespace-stripped) to EXPECTED_ANSWER.
mkdir -p /logs/verifier
AGENT_ANSWER=$(tr -d "[:space:]" < /output/agent/stdout.log 2>/dev/null)
EXPECTED=$(echo "$EXPECTED_ANSWER" | tr -d "[:space:]")
[ "$AGENT_ANSWER" = "$EXPECTED" ] && echo 1 > /logs/verifier/reward.txt || echo 0 > /logs/verifier/reward.txt
