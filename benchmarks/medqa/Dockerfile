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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/GBaker/MedQA-USMLE-4-options/parquet/default/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    opts = t['options'][i].as_py()
    question = str(t['question'][i])
    choices = '\n'.join(f'{k}. {opts[k]}' for k in ['A', 'B', 'C', 'D'])
    open(f'/tasks/{i}/problem.txt', 'w').write(f'{question}\n\n{choices}')
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answer_idx'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=medqa

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this USMLE-style medical question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
