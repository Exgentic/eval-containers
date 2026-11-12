#!/bin/bash
# Seed TASK from upstream's instruction if the runner didn't set it, and hand the
# repo to the agent uid (the base image builds it as root). Task IDENTITY is not
# exported — the shared run-agent strips it from the agent env (rule 7).
if [ -z "${TASK:-}" ] && [ -f /task/instruction.md ]; then
  # Assign then export (SC2155): an unreadable instruction must not silently
  # become an empty task.
  TASK="$(cat /task/instruction.md)"
  export TASK
fi
REPO=/app; [ -d /app ] || REPO=/testbed
chown -R agent:agent "$REPO" 2>/dev/null || true
exec "$@"
