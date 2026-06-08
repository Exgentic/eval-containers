# AgentBench (THUDM)
# AgentBench is a multi-suite agent benchmark across 8 environments.
# We package the DBBench standard split (300 tabular question-answering tasks)
# as a self-contained subset that does not require external services.
# Data: github THUDM/AgentBench (data/dbbench/standard.jsonl)
# Agent prints the answer to stdout. Test compares stdout to expected label.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-github:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="agentbench"
LABEL eval.benchmark.description="AgentBench (DBBench subset) - LLM-as-agent evaluation on tabular database questions"
LABEL eval.benchmark.tasks="300"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="d1e4a10db08c87075c78972e48ecc182be03e2d5"
LABEL eval.benchmark.url="https://github.com/THUDM/AgentBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2308.03688"

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
ENV EVAL_BENCHMARK=agentbench

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a database agent. Answer the following question using the provided table. Print ONLY the final answer value, with no explanation, no units, and no extra text.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
