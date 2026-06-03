# MedQA (GBaker/MedQA-USMLE-4-options)
# 1273 tasks. Data: HuggingFace GBaker/MedQA-USMLE-4-options (test split)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="medqa"
LABEL eval.benchmark.description="MedQA - USMLE-style medical licensing exam questions, 4 options"
LABEL eval.benchmark.tasks="1273"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18"
LABEL eval.benchmark.url="https://github.com/jind11/MedQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2009.13081"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
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
ENV EVAL_BENCHMARK=medqa

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this USMLE-style medical question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
