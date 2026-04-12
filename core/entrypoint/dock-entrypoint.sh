#!/bin/bash
set -euo pipefail

# Phase 1: Run the agent
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
timeout ${DOCK_TIMEOUT:-300} /opt/agent/entrypoint.sh || true
AGENT_EXIT=$?
ENDED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Write agent result
mkdir -p /output/agent /output/task /logs/verifier
printf '{"agent":"%s","started_at":"%s","ended_at":"%s","exit_code":%d}' \
  "$DOCK_AGENT" "$STARTED_AT" "$ENDED_AT" "$AGENT_EXIT" > /output/agent/result.json

# Phase 2: Run benchmark verification
bash /tests/test.sh

# Phase 3: Write task result
REWARD=$(cat /logs/verifier/reward.txt 2>/dev/null || echo 0)
PASSED=$([ "$REWARD" = "1" ] && echo true || echo false)
printf '{"task_id":"%s","benchmark":"%s","reward":%s,"passed":%s}' \
  "$TASK_ID" "$BENCHMARK" "$REWARD" "$PASSED" > /output/task/result.json
