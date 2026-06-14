#!/usr/bin/env python3
"""Regenerate task-profiles.json — task id → the MCP service sidecars it needs
— from the pinned EnterpriseOps-Gym dataset.

Each task's `gym_servers_config` lists the upstream gym_server_name fields
(e.g. `gym-csm-mcp`); we emit the DNS-1123 service names (`csm`) that the
chart's sidecars catalog and the compose service definitions both use — one
naming convention end to end. See .agents/benchmarks/RULES.md 24h. Re-run
when DATASET_REV changes:

    python3 containers/benchmarks/enterpriseops-gym/gen-task-profiles.py
"""

import io
import json
import urllib.request

import pyarrow.parquet as pq

# Pin: the EnterpriseOps-Gym HF dataset commit the map is derived from. Keep
# in sync with the BENCHMARK_VERSION the Dockerfile is built against.
DATASET_REV = "c8e538eae8a6205294f0a86675fefdc1fac408f6"
BASE = f"https://huggingface.co/datasets/ServiceNow-AI/EnterpriseOps-Gym/resolve/{DATASET_REV}/oracle/"
DOMAINS = ["calendar", "csm", "drive", "email", "hr", "hybrid", "itsm", "teams"]

# Upstream gym_server_name → DNS-1123 sidecar name. Keep in sync with the
# sidecars catalog in containers/benchmarks/_chart/presets/enterpriseops-gym.yaml.
GYM_TO_SIDECAR = {
    "gym-calendar": "calendar",
    "sn-csm-server": "csm",
    "gym-google-drive-mcp": "drive",
    "gym-email-mcp": "email",
    "sn-hr-internal": "hr",
    "gym-itsm-mcp": "itsm",
    "gym-teams-mcp": "teams",
}

profiles: dict[str, list[str]] = {}
index = 0
for domain in DOMAINS:
    url = f"{BASE}{domain}-00000-of-00001.parquet"
    table = pq.read_table(io.BytesIO(urllib.request.urlopen(url).read()))  # noqa: S310
    for row in table.to_pylist():
        servers = json.loads(row["gym_servers_config"])
        names = sorted(
            {
                GYM_TO_SIDECAR[s["mcp_server_name"]]
                for s in servers
                if s.get("mcp_server_name") in GYM_TO_SIDECAR
            }
        )
        # Sequential integer keys: same iteration order as the Dockerfile's
        # /tasks/all.jsonl materialization, so EVAL_TASK_ID = line index.
        # The upstream task_id is preserved in /tasks/<n>/id.txt at runtime
        # (RULES.md §15).
        profiles[str(index)] = names
        index += 1

items = sorted(profiles.items(), key=lambda kv: int(kv[0]))
body = (
    "{\n"
    + ",\n".join(f"  {json.dumps(k)}: {json.dumps(v)}" for k, v in items)
    + "\n}\n"
)

# Output lives in the chart so Helm's Files.Get can read it.
# Run from repo root.
with open(
    "containers/benchmarks/_chart/task-profiles/enterpriseops-gym.json", "w"
) as f:
    f.write(body)
print(f"wrote {len(items)} tasks from enterpriseops-gym@{DATASET_REV[:7]}")
