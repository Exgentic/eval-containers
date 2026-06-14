#!/usr/bin/env python3
"""Regenerate task-profiles.json — task id -> the site(s) it needs — from the
pinned WebArena Verified dataset.

The Helm chart self-resolves a task's site sidecars from this map (one Deployment+
Service per active site); compose reads the same map to name the task's services.
Site labels are emitted as DNS-1123 service names. See doctrine/benchmarks/RULES.md
24h. Re-run when DATASET_REV changes:

    python3 benchmarks/webarena/gen-task-profiles.py
"""

import json
import urllib.request

# Pin: the WebArena Verified dataset commit the map is derived from. Keep this in
# sync with the dataset revision the benchmark image is built against.
DATASET_REV = "6473f72db5dcefc97b5725b59e734504edc28a21"
URL = (
    "https://raw.githubusercontent.com/ServiceNow/webarena-verified/"
    f"{DATASET_REV}/assets/dataset/webarena-verified.json"
)

with urllib.request.urlopen(URL) as resp:  # noqa: S310 (pinned github raw URL)
    tasks = json.load(resp)

# Site labels in the dataset use the upstream key (e.g. "shopping_admin"); emit the
# DNS-1123 service name ("shopping-admin") so the map is the one naming convention
# shared by the catalog, chart, and compose.
profiles = {
    str(t["task_id"]): sorted(s.replace("_", "-") for s in t["sites"]) for t in tasks
}
items = sorted(profiles.items(), key=lambda kv: int(kv[0]))
body = (
    "{\n"
    + ",\n".join(f"  {json.dumps(k)}: {json.dumps(v)}" for k, v in items)
    + "\n}\n"
)

# Output lives in the chart so Helm's `Files.Get` can read it and self-resolve the
# task's sidecars (benchmarks/_chart/templates/job.yaml). Run from the repo root.
with open("benchmarks/_chart/task-profiles/webarena.json", "w") as f:
    f.write(body)
print(f"wrote {len(items)} tasks from webarena-verified@{DATASET_REV[:7]}")
