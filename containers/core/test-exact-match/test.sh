#!/bin/bash
# Shared exact-match grader (→ /grade.sh in every exact-match benchmark).
# Fail-closed: pre-seed 0, raise to 1 only on a real match — a missing/empty
# stdout or empty EXPECTED_ANSWER stays 0, else a no-op matches ""=="" and passes.
mkdir -p /logs/verifier
echo 0 > /logs/verifier/reward.txt

EXPECTED=$(printf '%s' "$EXPECTED_ANSWER" | tr -d "[:space:]")
AGENT_ANSWER=""
[ -s /output/agent/stdout.log ] && AGENT_ANSWER=$(tr -d "[:space:]" < /output/agent/stdout.log)

if [ -n "$EXPECTED" ] && [ "$AGENT_ANSWER" = "$EXPECTED" ]; then
  echo 1 > /logs/verifier/reward.txt
fi
