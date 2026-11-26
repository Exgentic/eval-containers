#!/bin/bash
# Lay the task's workspace out at /code (docs/, rtl/, verif/, rundir/) exactly as
# upstream's agentic runner mounts it, hand it to the agent, and seed TASK.
TASK_DIR="/tasks/${EVAL_TASK_ID:-0}"
if [ -d "$TASK_DIR/context" ]; then
  mkdir -p /code/docs /code/rtl /code/verif /code/rundir
  cp -r "$TASK_DIR/context/." /code/
  chown -R 1002:0 /code && chmod -R u+rwX,g+rwX /code
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  # Upstream's prompt.json carries the row's `prompt` alone. The preamble is
  # upstream's own Claude Code agent wrapper's default workspace rules
  # (examples/claude-code-agent/agent.py, BENCHMARK_WORKSPACE_RULES), verbatim.
  TASK="Benchmark workspace rules:
- Work inside /code and use the task prompt plus repository files to infer the required deliverables.
- Modify only project files needed for the task. The benchmark does not provide additional expected solution paths beyond the task and repository content.
- Do not modify benchmark infrastructure or generated artifacts, including prompt.json, Docker compose files, run scripts, hidden/reference solution files, run directories, generated logs, or cache directories.
- Do not alter benchmark harness infrastructure or generated output to force a passing score. If the task asks for RTL, verification code, documentation, assertions, or testbench work, make those changes in normal project files.

Task:
$(cat "$TASK_DIR/problem.txt")"
  export TASK
fi
exec "$@"
