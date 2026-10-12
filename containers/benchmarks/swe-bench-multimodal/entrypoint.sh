#!/bin/bash
# Hand the repo to the agent and seed TASK from the baked issue. Around the
# issue the prompt states facts the agent cannot find out, never how to work.
chown -R agent:agent /testbed 2>/dev/null || true
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  # A container reports its host's CPUs, and jest sizes its worker pool from
  # that: 95 workers under a 2-CPU, 12 GB limit got a whole task OOM-killed.
  cpus=$(awk '$1 != "max" {printf "%d", ($1 + $2 - 1) / $2}' /sys/fs/cgroup/cpu.max 2>/dev/null)
  mem=$(awk '$1 != "max" {printf "%d", $1 / 1073741824}' /sys/fs/cgroup/memory.max 2>/dev/null)
  limits=""
  if [ -n "$cpus" ] && [ -n "$mem" ]; then
    limits=" This container is limited to $cpus CPUs and $mem GB of memory, whatever it reports, and going over the memory limit kills the whole task."
  fi
  TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

The issue's screenshots are image files under /problem_assets/; the issue text gives each one's path where it appears. The repository's own Node.js is not on your PATH: \`source /usr/local/nvm/nvm.sh\` loads it. Chrome is installed as \`google-chrome\`.$limits

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
