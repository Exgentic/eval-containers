#!/bin/bash
# Oracle gold for cvdp: apply the task's reference `patch` to the /code workspace.
# Each entry is one file's unified-diff hunks without headers (upstream's
# GitRepositoryManager adds `--- a/<f>` / `+++ b/<f>` the same way before
# applying). The public dataset revision REDACTS the patch field, so this fails
# loudly there; it works on a build whose dataset carries solutions (Si2 LBC).
# Runs as root in place of the agent; /tasks is never agent-readable.
set -euo pipefail
PATCHES="/tasks/${EVAL_TASK_ID:-0}/patch"
if [ ! -d "$PATCHES" ]; then
  echo "cvdp oracle: no reference solution in this dataset revision (redacted upstream)" >&2
  exit 1
fi
cd "$PATCHES"
find . -type f | sed 's|^\./||' | while read -r f; do
  mkdir -p "/code/$(dirname "$f")"
  { printf -- '--- a/%s\n+++ b/%s\n' "$f" "$f"; cat "$f"; echo; } \
    | patch -p1 -d /code --forward --no-backup-if-mismatch
done
