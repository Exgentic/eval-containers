# DROP (Discrete Reasoning Over Paragraphs)
# 9535 tasks. Data: HuggingFace ucinlp/drop (validation)
# Agent prints answer to stdout. Test compares to first reference answer span.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="drop"
LABEL eval.benchmark.description="DROP - reading comprehension requiring discrete reasoning (counting, arithmetic, sorting) over paragraphs"
LABEL eval.benchmark.tasks="9535"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="95cda593fae71b60b5b19f82de3fcf3298c1239c"
LABEL eval.benchmark.url="https://huggingface.co/datasets/ucinlp/drop"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1903.00161"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="95cda593fae71b60b5b19f82de3fcf3298c1239c"

RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/ucinlp/drop/resolve/95cda593fae71b60b5b19f82de3fcf3298c1239c/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        passage = t['passage'][i].as_py()
        question = t['question'][i].as_py()
        spans = t['answers_spans'][i].as_py()['spans']
        answer = spans[0] if spans else ''
        row = {
            'id': str(t['query_id'][i].as_py()),
            'problem': f"Passage:\n{passage}\n\nQuestion: {question}",
            'answer': answer,
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(t)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=drop

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the passage and answer the question. Print only the final answer with no explanation. For numeric answers, print only the number. For span answers, print the exact text from the passage.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
