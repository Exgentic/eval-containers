# OpenBookQA (allenai/openbookqa)
# 500 tasks. Data: HuggingFace allenai/openbookqa (main/test split)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="openbookqa"
LABEL dock.benchmark.description="OpenBookQA - elementary science multiple choice"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="388097ea7776314e93a529163e0fea805b8a6454"
LABEL dock.benchmark.url="https://allenai.org/data/open-book-qa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1809.02789"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/openbookqa/resolve/388097ea7776314e93a529163e0fea805b8a6454/main/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['id'][i]))
    choices = t['choices'][i].as_py()
    stem = t['question_stem'][i].as_py()
    lines = [stem, ""]
    for label, text in zip(choices['label'], choices['text']):
        lines.append(f"{label}. {text}")
    open(f'/tasks/{i}/problem.txt', 'w').write("\n".join(lines))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answerKey'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=openbookqa

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice science question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
