# DROP (Discrete Reasoning Over Paragraphs)
# 9535 tasks. Data: HuggingFace ucinlp/drop (validation)
# Agent prints answer to stdout. Test compares to first reference answer span.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="drop"
LABEL dock.benchmark.description="DROP - reading comprehension requiring discrete reasoning (counting, arithmetic, sorting) over paragraphs"
LABEL dock.benchmark.tasks="9535"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="95cda593fae71b60b5b19f82de3fcf3298c1239c"
LABEL dock.benchmark.url="https://huggingface.co/datasets/ucinlp/drop"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1903.00161"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/ucinlp/drop/resolve/95cda593fae71b60b5b19f82de3fcf3298c1239c/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['query_id'][i].as_py()))
    passage = t['passage'][i].as_py()
    question = t['question'][i].as_py()
    spans = t['answers_spans'][i].as_py()['spans']
    answer = spans[0] if spans else ''
    open(f'/tasks/{i}/problem.txt', 'w').write(f"Passage:\n{passage}\n\nQuestion: {question}")
    open(f'/tasks/{i}/answer.txt', 'w').write(answer)
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=drop

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the passage and answer the question. Print only the final answer with no explanation. For numeric answers, print only the number. For span answers, print the exact text from the passage.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
