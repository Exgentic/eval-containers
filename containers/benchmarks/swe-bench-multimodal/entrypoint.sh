#!/bin/bash
# Hand the repo to the agent and seed TASK from the baked issue.
chown -R agent:agent /testbed 2>/dev/null || true
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

The images and pages the issue links to are saved under /problem_assets/ — look at them. The repository's own Node.js is not on your PATH: run \`source /usr/local/nvm/nvm.sh\` before any project command.

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
