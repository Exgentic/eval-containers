# OpenBookQA (allenai/openbookqa)
# 500 tasks. Data: HuggingFace allenai/openbookqa (main/test split)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="openbookqa"
LABEL eval.benchmark.description="OpenBookQA - elementary science multiple choice"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="388097ea7776314e93a529163e0fea805b8a6454"
LABEL eval.benchmark.url="https://allenai.org/data/open-book-qa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1809.02789"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="388097ea7776314e93a529163e0fea805b8a6454"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/openbookqa/resolve/388097ea7776314e93a529163e0fea805b8a6454/main/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        choices = t['choices'][i].as_py()
        stem = t['question_stem'][i].as_py()
        lines = [stem, ""]
        for label, text in zip(choices['label'], choices['text']):
            lines.append(f"{label}. {text}")
        row = {
            'id': str(t['id'][i]),
            'problem': "\n".join(lines),
            'answer': str(t['answerKey'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=openbookqa

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice science question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
