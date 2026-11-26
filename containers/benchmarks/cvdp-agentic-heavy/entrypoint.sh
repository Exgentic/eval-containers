#!/bin/bash
# Build the task's workspace at /code the way upstream's context-heavy runner does
# (git_utils _run_patch_container): check out the sanitized context commit from
# the task's git bundle, then `cp -a ./<root>/. /code/` where <root> is
# `external` (or `.` for `_ext_` repos). Hand it to the agent and seed TASK.
TASK_DIR="/tasks/${EVAL_TASK_ID:-0}"
if [ -f "$TASK_DIR/repo.txt" ]; then
  repo=$(cat "$TASK_DIR/repo.txt"); commit=$(cat "$TASK_DIR/commit.txt"); root=$(cat "$TASK_DIR/root.txt")
  # On disk, not /tmp: the shared runner mounts /tmp as a 1G tmpfs, and the
  # largest checkout (SoomRV) is 2.4G. Same filesystem as /code, so the subtree
  # moves into place by rename instead of a second full copy.
  ws=$(mktemp -d /var/tmp/cvdp-ws.XXXXXX)
  git -c init.defaultBranch=main -C "$ws" init -q \
    && git -C "$ws" fetch -q "/tasks/bundles/$repo.bundle" "$commit" \
    && git -C "$ws" -c advice.detachedHead=false checkout -q "$commit" \
    || { echo "cvdp-agentic-heavy: cannot check out $repo@$commit" >&2; exit 1; }
  # /code is the image's WORKDIR: leave it before replacing it, and come back,
  # so the agent (which inherits this cwd) starts in the new workspace.
  # Upstream copies `<root>/.` (cp -a); moving that directory is the same tree.
  cd / && rm -rf /code && mkdir -p /rundir/harness
  if [ "$root" = "." ]; then mv "$ws" /code; else mv "$ws/$root" /code && rm -rf "$ws"; fi
  chown -R 1002:0 /code && chmod -R u+rwX,g+rwX /code
  cd /code || exit 1
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
