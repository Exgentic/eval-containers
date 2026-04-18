# OpenBookQA (allenai/openbookqa)
# 500 tasks. Data: HuggingFace allenai/openbookqa (main/test split)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="openbookqa"
LABEL dock.benchmark.description="OpenBookQA - elementary science multiple choice"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="388097ea7776314e93a529163e0fea805b8a6454"
LABEL dock.benchmark.url="https://allenai.org/data/open-book-qa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1809.02789"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="388097ea7776314e93a529163e0fea805b8a6454"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
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
ENV DOCK_BENCHMARK=openbookqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice science question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
