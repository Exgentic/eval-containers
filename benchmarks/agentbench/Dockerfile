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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="d1e4a10db08c87075c78972e48ecc182be03e2d5"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch the DBBench standard split and write every task to /tasks/all.jsonl.
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "d1e4a10db08c87075c78972e48ecc182be03e2d5"
URL = f"https://raw.githubusercontent.com/THUDM/AgentBench/{SHA}/data/dbbench/standard.jsonl"
data = urllib.request.urlopen(URL).read().decode("utf-8")

os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
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

        out_row = {
            'id': f'dbbench/{idx}',
            'source': str(row.get('source', '')),
            'problem': description + "\n\n" + table_str,
            'answer': str(answer),
        }
        out.write(json.dumps(out_row) + '\n')
        idx += 1
print(f"Extracted {idx} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=agentbench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a database agent. Answer the following question using the provided table. Print ONLY the final answer value, with no explanation, no units, and no extra text.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
