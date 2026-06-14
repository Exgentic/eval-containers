# EnterpriseOps-Gym (ServiceNow AI Research, 2026)
# 649 single-agent enterprise tasks across 8 domains (calendar, csm, drive,
# email, hr, hybrid, itsm, teams). Outcome-based scoring: SQL verifiers query
# the post-run database via each MCP server's /api/sql-runner HTTP endpoint.
# Deterministic — no LLM-as-judge in the hot path.
#
# Tool-set mode: this image ships oracle (only the exact tools each task
# needs). plus_5_tools / plus_10_tools / plus_15_tools are future versions.
#
# Upstream: github.com/ServiceNow/EnterpriseOps-Gym
# Dataset:  huggingface.co/datasets/ServiceNow-AI/EnterpriseOps-Gym
# Paper:    arxiv.org/abs/2603.13594

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

# Rule 4: a single ARG drives both the fetch step and the data_revision label,
# so the two can never disagree. Bake into ENV so the (quoted) Python heredocs
# below read via os.environ — hadolint doesn't parse unquoted heredocs.
ARG BENCHMARK_VERSION=c8e538eae8a6205294f0a86675fefdc1fac408f6
ARG UPSTREAM_REPO_SHA=09593147ac79cf722cf2111ac6d4b10b9a6d5a58
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}
ENV UPSTREAM_REPO_SHA=${UPSTREAM_REPO_SHA}

LABEL eval.type="benchmark"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.name="enterpriseops-gym"
LABEL eval.benchmark.description="EnterpriseOps-Gym - stateful enterprise agent tasks over MCP, SQL-verified"
LABEL eval.benchmark.tasks="649"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/ServiceNow/EnterpriseOps-Gym"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2603.13594"
LABEL eval.benchmark.dataset="https://huggingface.co/datasets/ServiceNow-AI/EnterpriseOps-Gym"
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"

# ── Tasks: oracle-mode parquets, rewritten for the internal arena ─────
# Pull all 8 domain parquets from HF (pinned via $BENCHMARK_VERSION),
# concatenate into /tasks/all.jsonl, and rewrite the upstream gym_server
# URLs from http://localhost:<host-port> to the in-cluster service names
# the agent will actually reach.
RUN python3 <<'PYEOF'
import urllib.request, json, os, io
import pyarrow.parquet as pq

HF_SHA = os.environ["BENCHMARK_VERSION"]
BASE = f"https://huggingface.co/datasets/ServiceNow-AI/EnterpriseOps-Gym/resolve/{HF_SHA}/oracle/"
DOMAINS = ["calendar", "csm", "drive", "email", "hr", "hybrid", "itsm", "teams"]

# Upstream gym_server_name → in-cluster service hostname:port. Hostnames match
# the compose service / chart sidecar names; ports are the upstream container
# ports (8005 everywhere except calendar's 8003).
GYM_HOSTS = {
    "gym-calendar":         "http://calendar:8003",
    "sn-csm-server":        "http://csm:8005",
    "gym-google-drive-mcp": "http://drive:8005",
    "gym-email-mcp":        "http://email:8005",
    "sn-hr-internal":       "http://hr:8005",
    "gym-itsm-mcp":         "http://itsm:8005",
    "gym-teams-mcp":        "http://teams:8005",
}

os.makedirs("/tasks", exist_ok=True)
total = 0
with open("/tasks/all.jsonl", "w") as out:
    for domain in DOMAINS:
        url = f"{BASE}{domain}-00000-of-00001.parquet"
        data = urllib.request.urlopen(url).read()
        table = pq.read_table(io.BytesIO(data))
        for row in table.to_pylist():
            servers = json.loads(row["gym_servers_config"])
            for s in servers:
                name = s.get("mcp_server_name")
                if name in GYM_HOSTS:
                    s["mcp_server_url"] = GYM_HOSTS[name]
            out.write(json.dumps({
                "id":                  row["task_id"],
                "domain":              row["domain"],
                "system_prompt":       row["system_prompt"],
                "user_prompt":         row["user_prompt"],
                "selected_tools":      json.dumps(list(row["selected_tools"] or [])),
                "restricted_tools":    json.dumps(list(row["restricted_tools"] or [])),
                "gym_servers_config":  json.dumps(servers),
                "verifiers":           row["verifiers"],
                "problem":             row["user_prompt"],
            }) + "\n")
            total += 1
assert total == 649, f"expected 649 tasks at pinned dataset SHA, got {total}"
print(f"Wrote {total} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

# ── Seed SQL corpus from upstream gym_dbs.zip ─────────────────────────
# ~14 MB of per-task seed databases. Root-only per §5: the agent (non-root)
# must not see the seed corpus; test.sh (root) reads it during pre-agent
# setup to create fresh DB instances on the MCP services.
RUN python3 <<'PYEOF'
import urllib.request, zipfile, io, os
URL = f"https://raw.githubusercontent.com/ServiceNow/EnterpriseOps-Gym/{os.environ['UPSTREAM_REPO_SHA']}/gym_dbs.zip"
buf = io.BytesIO(urllib.request.urlopen(URL).read())
with zipfile.ZipFile(buf) as zf:
    zf.extractall("/seed")
print("gym_dbs.zip extracted into /seed")
PYEOF
RUN find /seed -type d -exec chmod 700 {} + && \
    find /seed -type f -exec chmod 600 {} +

WORKDIR /app
ENV EVAL_BENCHMARK=enterpriseops-gym

# Pre-agent: build /var/eval-state/{state.json, task.txt} for the agent.
# Post-agent: query final DB state and compute reward.
COPY setup_task.py /app/setup_task.py
COPY verify_task.py /app/verify_task.py
RUN chmod 755 /app/setup_task.py /app/verify_task.py

# Replace the inherited reward=-1 grader with the real verifier runner.
RUN cat > /grade.sh <<'TEST'
#!/bin/bash
python3 /app/verify_task.py
TEST
RUN chmod +x /grade.sh

# Entrypoint: materialize task → create per-task DBs and build TASK string → exec the shared eval entrypoint.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
set -e
/eval-materialize-task
python3 /app/setup_task.py
export TASK="$(cat /var/eval-state/task.txt)"
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
