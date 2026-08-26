#!/bin/bash
# Oracle gold for hwe-bench: apply the dataset's gold `fix_patch` to the RTL repo.
# grade.sh then captures it as the candidate diff and grade.py runs tb_script,
# scoring resolved=1. The gold ships in /tasks/0/config.json (root-only, chmod
# 600); the oracle runs as root so it can read it — never readable by the
# sandboxed agent. Mirrors swe-bench/solution.sh. See core/oracle/README.md.
set -euo pipefail
REPO="${HWE_REPO_DIR:-/home}"
cd "$REPO"
git config --global --add safe.directory "$REPO" 2>/dev/null || true
python3 -c "import json,sys; sys.stdout.write(json.load(open('/tasks/0/config.json'))['fix_patch'])" | git apply -v
