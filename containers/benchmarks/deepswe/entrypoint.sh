#!/bin/bash
# Seed TASK from the baked instruction if the runner didn't set it, and make the
# repo writable by the agent uid (the base image builds /app as root).
# Task IDENTITY is deliberately not exported here — the shared run-agent strips it
# from the agent env (rule 7); only the problem text reaches the agent.
if [ -z "${TASK:-}" ] && [ -f /task/instruction.md ]; then
  export TASK="$(cat /task/instruction.md)"
fi
chown -R agent:agent /app 2>/dev/null || true
exec "$@"
