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
# Apply only the SOURCE hunks of the gold patch. Some cases' fix_patch also bumps
# a submodule pointer (a "diff --git" block with mode 160000 / "Subproject commit"
# lines). The agent's contract is to edit tracked superproject HDL source only, so
# grade.sh captures the candidate with --ignore-submodules=all — the oracle gold
# must match that contract, or the submodule hunk fails `git apply` atomically
# (set -e) and the whole gold is dropped, spuriously scoring the case 0. Filtering
# to source-only keeps oracle and live grading symmetric; a case whose fix lives
# ONLY in a submodule filters to empty here and is out of scope (not in tasks.txt).
python3 - <<'PY' | git apply -v
import json, re, sys
fp = json.load(open('/tasks/0/config.json'))['fix_patch']
# Plain patches (no submodule hunk) pass through verbatim — byte-identical to a
# straight `git apply`, so source-only cases are unaffected. Only when a submodule
# hunk is present do we split per-file and drop the submodule blocks.
if 'Subproject commit' in fp or '160000' in fp:
    blocks = re.split(r'(?m)^(?=diff --git )', fp)
    fp = ''.join(b for b in blocks if b and not ('160000' in b or 'Subproject commit' in b))
sys.stdout.write(fp)
PY
