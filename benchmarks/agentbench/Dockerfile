# AgentBench (THUDM)
# AgentBench is a multi-suite agent benchmark across 8 environments.
# We package the DBBench standard split (300 tabular question-answering tasks)
# as a self-contained subset that does not require external services.
# Data: github THUDM/AgentBench (data/dbbench/standard.jsonl)
# Agent prints the answer to stdout. Test compares stdout to expected label.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="agentbench"
LABEL dock.benchmark.description="AgentBench (DBBench subset) - LLM-as-agent evaluation on tabular database questions"
LABEL dock.benchmark.tasks="300"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="d1e4a10db08c87075c78972e48ecc182be03e2d5"
LABEL dock.benchmark.url="https://github.com/THUDM/AgentBench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2308.03688"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch the DBBench standard split and flatten to /tasks/<idx>/{problem,answer,id}.txt
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "d1e4a10db08c87075c78972e48ecc182be03e2d5"
URL = f"https://raw.githubusercontent.com/THUDM/AgentBench/{SHA}/data/dbbench/standard.jsonl"
data = urllib.request.urlopen(URL).read().decode("utf-8")

idx = 0
for line in data.splitlines():
    line = line.strip()
    if not line:
        continue
    row = json.loads(line)
    description = row.get("description", "")
    table = row.get("table", {}) or {}
    table_name = table.get("table_name", "")
    info = table.get("table_info", {}) or {}
    columns = info.get("columns", []) or []
    rows = info.get("rows", []) or []
    col_names = [c.get("name", "") for c in columns]
    lines = [f"Table: {table_name}", "Columns: " + ", ".join(col_names), "Rows:"]
    for r in rows:
        lines.append("  " + " | ".join(str(v) for v in r))
    table_str = "\n".join(lines)

    label = row.get("label", [])
    if isinstance(label, list):
        answer = label[0] if label else ""
    else:
        answer = str(label)

    d = f"/tasks/{idx}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(f"dbbench/{idx}")
    open(f"{d}/source.txt", "w").write(str(row.get("source", "")))
    open(f"{d}/problem.txt", "w").write(description + "\n\n" + table_str)
    open(f"{d}/answer.txt", "w").write(str(answer))
    idx += 1
print(f"Extracted {idx} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=agentbench

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a database agent. Answer the following question using the provided table. Print ONLY the final answer value, with no explanation, no units, and no extra text.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
