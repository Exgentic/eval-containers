#!/bin/bash
# Oracle for swe-bench (Epoch / official-harness flow): apply the gold `patch` to
# /testbed. The grader captures the working tree via `git diff` as the prediction
# and the swebench harness scores resolved=1. The gold patch ships in
# /tasks/0/config.json (root-only); the oracle runs as root, so it can read it —
# it is never readable by the sandboxed agent.
set -euo pipefail
cd /testbed
git config --global --add safe.directory /testbed 2>/dev/null || true
python3 -c "import json,sys; sys.stdout.write(json.load(open('/tasks/0/config.json'))['patch'])" | git apply -v
