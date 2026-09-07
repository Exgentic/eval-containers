#!/bin/bash
# Seed TASK from the baked instruction if the runner didn't set it, and make the
# repo writable by the agent uid (the base image builds /app as root).
# Task IDENTITY is deliberately not exported here — the shared run-agent strips it
# from the agent env (rule 7); only the problem text reaches the agent.
if [ -z "${TASK:-}" ] && [ -f /task/instruction.md ]; then
  # Assign then export (SC2155): `export TASK="$(cat …)"` masks cat's exit status,
  # so an unreadable instruction would silently yield an empty task rather than fail.
  TASK="$(cat /task/instruction.md)"
  export TASK
fi
chown -R agent:agent /app 2>/dev/null || true
exec "$@"
