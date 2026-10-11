#!/bin/bash
# Hand the repo to the agent and seed TASK from the baked issue.
chown -R agent:agent /testbed 2>/dev/null || true
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  # A container reports its host's CPUs, and jest and friends size their worker
  # pools from that: on a 96-core node one test run started 95 workers under a
  # 2-CPU, 12 GB limit and the whole task was OOM-killed. Say what the limit is.
  cpus=$(awk '$1 != "max" {printf "%d", ($1 + $2 - 1) / $2}' /sys/fs/cgroup/cpu.max 2>/dev/null)
  mem=$(awk '$1 != "max" {printf "%d", $1 / 1073741824}' /sys/fs/cgroup/memory.max 2>/dev/null)
  limits=""
  if [ -n "$cpus" ] && [ -n "$mem" ]; then
    limits=" This container is limited to $cpus CPUs and $mem GB of memory, whatever it reports: cap the parallelism of any test or build you run (for example \`--maxWorkers=$cpus\`), or the task is killed."
  fi
  TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

The issue's screenshots are image files under /problem_assets/, named in the issue text where each one appears. Open and look at every one of them before you change any code. To see what your change renders, Chrome is installed: \`google-chrome --headless --no-sandbox --screenshot=/tmp/shot.png <file or URL>\` writes an image you can open.

The repository's own Node.js is not on your PATH: run \`source /usr/local/nvm/nvm.sh\` before any project command.$limits

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
