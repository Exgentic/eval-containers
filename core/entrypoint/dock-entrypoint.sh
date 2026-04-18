#!/bin/bash
# Shared evaluation entrypoint.
#
# Implements RULES.md principle 9 (two orthogonal knobs: container tag vs
# internal upstream version) on the benchmark + agent axes. Container tag is
# selected at `docker pull` time by `DOCK_BENCHMARK_TAG` / `DOCK_AGENT_TAG` /
# `DOCK_MODEL_TAG` (Docker's job — not this script's). Internal upstream
# version is read from:
#
#   DOCK_BENCHMARK_VERSION  (dataset revision; overrides DOCK_BENCHMARK_VERSION_DEFAULT)
#   DOCK_AGENT_VERSION      (upstream CLI version; overrides DOCK_AGENT_VERSION_DEFAULT)
#
# If a version override is set AND differs from the baked default, this script
# invokes an opt-in reloader hook (`/dock-refetch-data` for benchmarks,
# `/dock-reinstall-agent` for agents) that the image may ship. If no hook is
# present but an override is set, the run fails loud rather than silently
# running the wrong version.
#
# In all cases, the resolved version is recorded to /output/task/version.json
# (benchmark) and /output/agent/version.json (agent) before the agent runs.
set -euo pipefail

# Create agent user if needed
id agent &>/dev/null || useradd -m -s /bin/bash agent

# Prepare directories
mkdir -p /output/agent /output/task /logs/verifier /app
chown -R agent:agent /output/agent /logs /app /tmp 2>/dev/null || true

# ─── Benchmark version resolution (rule 9) ────────────────────────
BENCH_DEFAULT="${DOCK_BENCHMARK_VERSION_DEFAULT:-}"
BENCH_OVERRIDE="${DOCK_BENCHMARK_VERSION:-}"
BENCH_RESOLVED="${BENCH_OVERRIDE:-$BENCH_DEFAULT}"
if [ -n "$BENCH_OVERRIDE" ] && [ "$BENCH_OVERRIDE" != "$BENCH_DEFAULT" ]; then
  if [ -x /dock-refetch-data ]; then
    echo "dock: benchmark version override $BENCH_DEFAULT -> $BENCH_OVERRIDE" >&2
    /dock-refetch-data "$BENCH_OVERRIDE"
  else
    echo "dock: DOCK_BENCHMARK_VERSION=$BENCH_OVERRIDE set but this image has no /dock-refetch-data hook (baked default: $BENCH_DEFAULT). Refusing to run." >&2
    exit 64
  fi
fi
printf '{"benchmark":"%s","default":"%s","override":"%s","resolved":"%s"}' \
  "${DOCK_BENCHMARK:-unknown}" "$BENCH_DEFAULT" "$BENCH_OVERRIDE" "$BENCH_RESOLVED" \
  > /output/task/version.json

# ─── Agent version resolution (rule 9) ────────────────────────────
AGENT_DEFAULT="${DOCK_AGENT_VERSION_DEFAULT:-}"
AGENT_OVERRIDE="${DOCK_AGENT_VERSION:-}"
AGENT_RESOLVED="${AGENT_OVERRIDE:-$AGENT_DEFAULT}"
if [ -n "$AGENT_OVERRIDE" ] && [ "$AGENT_OVERRIDE" != "$AGENT_DEFAULT" ]; then
  if [ -x /dock-reinstall-agent ]; then
    echo "dock: agent version override $AGENT_DEFAULT -> $AGENT_OVERRIDE" >&2
    /dock-reinstall-agent "$AGENT_OVERRIDE"
  else
    echo "dock: DOCK_AGENT_VERSION=$AGENT_OVERRIDE set but this image has no /dock-reinstall-agent hook (baked default: $AGENT_DEFAULT). Refusing to run." >&2
    exit 64
  fi
fi
printf '{"agent":"%s","default":"%s","override":"%s","resolved":"%s"}' \
  "${DOCK_AGENT:-unknown}" "$AGENT_DEFAULT" "$AGENT_OVERRIDE" "$AGENT_RESOLVED" \
  > /output/agent/version.json

# ─── Preserve task input for inspection ──────────────────────────
# Copy the materialized task files into /output/task/input/ so every
# run artifact is self-describing — you can read what the agent was
# asked, the ground truth, and any attached files without needing the
# benchmark image. Used by the live-sweep driver for audit trails.
# See tests/live/RULES.md. Silent on failure because not every
# benchmark populates /tasks/$DOCK_TASK_ID (per-task-build images
# build the task into the image itself).
if [ -n "${DOCK_TASK_ID:-}" ] && [ -d "/tasks/$DOCK_TASK_ID" ]; then
  mkdir -p /output/task/input
  cp -r "/tasks/$DOCK_TASK_ID/." /output/task/input/ 2>/dev/null || true
fi

# Hide expected answer from agent
SAVED_EXPECTED_ANSWER="${EXPECTED_ANSWER:-}"
unset EXPECTED_ANSWER

# Phase 1: Run agent as non-root, capture stdout/stderr
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
su agent -s /bin/bash -c "
  export TASK='$(echo "$TASK" | sed "s/'/'\\\\''/g")'
  export DOCK_TASK_ID='${DOCK_TASK_ID:-}'
  export DOCK_MODEL='${DOCK_MODEL:-}'
  export OPENAI_BASE_URL='${OPENAI_BASE_URL:-}'
  export OPENAI_API_KEY='${OPENAI_API_KEY:-}'
  export ANTHROPIC_BASE_URL='${ANTHROPIC_BASE_URL:-}'
  export ANTHROPIC_API_KEY='${ANTHROPIC_API_KEY:-}'
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
# Numeric comparison so "1.0" == 1 (graders that emit floats still resolve
# to passed=true when they hit perfect score). `bc` is available in every
# benchmark base; fall back to python if missing.
PASSED=$(awk -v r="$REWARD" 'BEGIN{print (r+0 >= 1) ? "true" : "false"}')
printf '{"task_id":"%s","benchmark":"%s","reward":%s,"passed":%s}' \
  "${DOCK_TASK_ID:-unknown}" "${DOCK_BENCHMARK:-unknown}" "$REWARD" "$PASSED" > /output/task/result.json
