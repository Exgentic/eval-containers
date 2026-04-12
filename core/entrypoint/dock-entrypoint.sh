#!/bin/bash
set -euo pipefail

# Create agent user if it doesn't exist
id agent &>/dev/null || useradd -m -s /bin/bash agent

# Load task from tasks.json if TASK is not set
if [ -z "${TASK:-}" ] && [ -f /tasks/tasks.json ] && [ -n "${TASK_ID:-}" ]; then
  TASK=$(python3 -c "import json; d=json.load(open('/tasks/tasks.json')); print(d['${TASK_ID}']['instruction'])")
  EXPECTED_ANSWER=$(python3 -c "import json; d=json.load(open('/tasks/tasks.json')); print(d['${TASK_ID}']['expected_answer'])")
  export TASK EXPECTED_ANSWER
fi

# Save answer for verification, clear from environment
SAVED_EXPECTED_ANSWER="${EXPECTED_ANSWER:-}"
unset EXPECTED_ANSWER

# Write task instruction to a file the agent can read
mkdir -p /output/agent /output/task /logs/verifier /app
echo "$TASK" > /app/task.txt
chown -R agent:agent /output/agent /logs /app /tmp 2>/dev/null || true

# Write agent env file (only what the agent needs)
cat > /tmp/agent.env <<ENVEOF
export TASK="$(cat /app/task.txt)"
export TASK_ID="${TASK_ID:-}"
export DOCK_TIMEOUT="${DOCK_TIMEOUT:-300}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-}"
export ANTHROPIC_BASE_URL="${ANTHROPIC_BASE_URL:-}"
ENVEOF
chown agent:agent /tmp/agent.env

# Phase 1: Run the agent as non-root user
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
su agent -s /bin/bash -c "source /tmp/agent.env && timeout \${DOCK_TIMEOUT:-300} /opt/agent/entrypoint.sh" || true
AGENT_EXIT=$?
ENDED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Write agent result (as root)
printf '{"agent":"%s","started_at":"%s","ended_at":"%s","exit_code":%d}' \
  "${DOCK_AGENT:-unknown}" "$STARTED_AT" "$ENDED_AT" "$AGENT_EXIT" > /output/agent/result.json

# Phase 2: Run benchmark verification (as root, with answer restored)
export EXPECTED_ANSWER="$SAVED_EXPECTED_ANSWER"
bash /tests/test.sh

# Phase 3: Write task result
REWARD=$(cat /logs/verifier/reward.txt 2>/dev/null || echo 0)
PASSED=$([ "$REWARD" = "1" ] && echo true || echo false)
printf '{"task_id":"%s","benchmark":"%s","reward":%s,"passed":%s}' \
  "${TASK_ID:-unknown}" "${BENCHMARK:-unknown}" "$REWARD" "$PASSED" > /output/task/result.json
