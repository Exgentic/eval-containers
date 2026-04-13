#!/bin/bash
set -euo pipefail

# Create agent user if needed
id agent &>/dev/null || useradd -m -s /bin/bash agent

# Prepare directories
mkdir -p /output/agent /output/task /logs/verifier /app
chown -R agent:agent /output/agent /logs /app /tmp 2>/dev/null || true

# Hide expected answer from agent
SAVED_EXPECTED_ANSWER="${EXPECTED_ANSWER:-}"
unset EXPECTED_ANSWER

# Phase 1: Run agent as non-root, capture stdout/stderr
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
su agent -s /bin/bash -c "
  export TASK='$(echo "$TASK" | sed "s/'/'\\\\''/g")'
  export TASK_ID='${TASK_ID:-}'
  export OPENAI_BASE_URL='${OPENAI_BASE_URL:-}'
  export ANTHROPIC_BASE_URL='${ANTHROPIC_BASE_URL:-}'
  timeout ${DOCK_TIMEOUT:-300} /opt/agent/entrypoint.sh
" > /output/agent/stdout.log 2> /output/agent/stderr.log || true
AGENT_EXIT=$?
ENDED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Write agent result
printf '{"agent":"%s","started_at":"%s","ended_at":"%s","exit_code":%d}' \
  "${DOCK_AGENT:-unknown}" "$STARTED_AT" "$ENDED_AT" "$AGENT_EXIT" > /output/agent/result.json

# Phase 2: Verify (as root, with answer restored)
export EXPECTED_ANSWER="$SAVED_EXPECTED_ANSWER"
bash /tests/test.sh || true

# Phase 3: Write task result
REWARD=$(cat /logs/verifier/reward.txt 2>/dev/null || echo 0)
PASSED=$([ "$REWARD" = "1" ] && echo true || echo false)
printf '{"task_id":"%s","benchmark":"%s","reward":%s,"passed":%s}' \
  "${TASK_ID:-unknown}" "${BENCHMARK:-unknown}" "$REWARD" "$PASSED" > /output/task/result.json
