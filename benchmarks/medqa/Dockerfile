# MedQA (GBaker/MedQA-USMLE-4-options)
# 1273 tasks. Data: HuggingFace GBaker/MedQA-USMLE-4-options (test split)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="medqa"
LABEL dock.benchmark.description="MedQA - USMLE-style medical licensing exam questions, 4 options"
LABEL dock.benchmark.tasks="1273"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18"
LABEL dock.benchmark.url="https://github.com/jind11/MedQA"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2009.13081"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/GBaker/MedQA-USMLE-4-options/parquet/default/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        opts = t['options'][i].as_py()
        question = str(t['question'][i])
        choices = '\n'.join(f'{k}. {opts[k]}' for k in ['A', 'B', 'C', 'D'])
        row = {
            'id': str(i),
            'problem': f'{question}\n\n{choices}',
            'answer': str(t['answer_idx'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=medqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this USMLE-style medical question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
