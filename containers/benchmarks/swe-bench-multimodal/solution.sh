#!/bin/bash
# Oracle gold for swe-bench-multimodal: apply the dataset's gold `patch` to
# /testbed the way the harness applies a model's — its own chain of fallbacks,
# because some images' trees already differ from the base commit and a plain
# `git apply` refuses. The patch ships in /tasks/0/config.json (root-only); the
# oracle runs as root, so it can read it — the sandboxed agent cannot.
set -euo pipefail
cd /testbed
/tests/python/bin/python3 - <<'PY'
import json
import subprocess
import sys

sys.path.insert(0, "/tests/deps")
from swebench.harness.run_evaluation import GIT_APPLY_CMDS

with open("/tmp/gold.diff", "w") as out:
    out.write(json.load(open("/tasks/0/config.json"))["patch"])
for attempt, command in enumerate(GIT_APPLY_CMDS):
    if attempt:  # a failed attempt leaves partial state behind
        subprocess.run("git checkout -- . ; git clean -fd", shell=True)
    if subprocess.run(f"{command} /tmp/gold.diff", shell=True).returncode == 0:
        sys.exit(0)
sys.exit(1)
PY
