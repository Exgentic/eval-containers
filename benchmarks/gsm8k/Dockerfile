# GSM8K (Grade School Math 8K)
# 1319 tasks. Data: HuggingFace openai/gsm8k (main/test)
# Agent prints answer to stdout. Test compares to expected numeric answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gsm8k"
LABEL dock.benchmark.description="GSM8K - grade school math word problems requiring multi-step reasoning"
LABEL dock.benchmark.tasks="1319"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="740312add88f781978c0658806c59bc2815b9866"
LABEL dock.benchmark.url="https://huggingface.co/datasets/openai/gsm8k"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2110.14168"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="740312add88f781978c0658806c59bc2815b9866"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Single-JSONL task layout (see benchmarks/RULES.md #22). One line per task,
# /dock-materialize-task materializes only the requested row at runtime.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/openai/gsm8k/parquet/main/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        full_answer = str(t['answer'][i])
        numeric = full_answer.split('####')[-1].strip().replace(',', '')
        row = {
            'id': str(i),
            'problem': str(t['question'][i]),
            'answer': numeric,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=gsm8k

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this grade school math problem. Print only the final numeric answer as a single number with no units, commas, or explanation.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
