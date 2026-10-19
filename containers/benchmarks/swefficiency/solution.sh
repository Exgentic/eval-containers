#!/bin/bash
# Oracle for swefficiency: apply the expert patch (the PR that fixed the task) to
# /testbed. It ships in /tasks/0/config.json, root-only, because the grader times
# it beside the agent's; the oracle runs as root, the agent never reads it.
set -euo pipefail
cd /testbed
git config --global --add safe.directory /testbed 2>/dev/null || true
/opt/miniconda3/bin/python -c "import json,sys; sys.stdout.write(json.load(open('/tasks/0/config.json'))['patch'])" | git apply -v
