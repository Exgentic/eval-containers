#!/bin/bash
# Oracle gold for swe-bench-pro: apply the dataset's gold `patch` to the repo at
# /app. grade.py then applies the test_patch and runs the selected tests, scoring
# resolved=1. The gold ships in /tasks/0/config.json (root-only, chmod 600); the
# oracle runs as root, so it can read it — never readable by the sandboxed agent.
# Mirrors swe-bench's solution.sh. See core/oracle/README.md.
set -euo pipefail
cd /app
git config --global --add safe.directory /app 2>/dev/null || true
python3 -c "import json,sys; sys.stdout.write(json.load(open('/tasks/0/config.json'))['patch'])" | git apply -v
