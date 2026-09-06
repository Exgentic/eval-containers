#!/bin/bash
# Hand the hardware-design repo to the agent and seed TASK from the problem statement.
REPO="${HWE_REPO_DIR:-/home}"
chown -R agent:agent "$REPO" 2>/dev/null || true
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK="Fix the hardware bug in the repository at ${REPO}. Edit the design source (SystemVerilog/Verilog RTL, or Chisel/Scala for the Chisel-based repos) so the failing test passes. Do NOT modify any test files or testbenches.

$(cat /tasks/0/problem.txt)"
  export TASK
fi
exec "$@"
