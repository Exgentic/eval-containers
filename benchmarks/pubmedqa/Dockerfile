# PubMedQA (qiaojin/PubMedQA)
# 1000 tasks. Data: HuggingFace qiaojin/PubMedQA (pqa_labeled/train)
# Agent reads a biomedical abstract and answers yes/no/maybe to a question.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="pubmedqa"
LABEL dock.benchmark.description="PubMedQA - biomedical research question answering from abstracts"
LABEL dock.benchmark.tasks="1000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="9001f2853fb87cab8d220904e0de81ac6973b318"
LABEL dock.benchmark.url="https://github.com/pubmedqa/pubmedqa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1909.06146"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="9001f2853fb87cab8d220904e0de81ac6973b318"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/qiaojin/PubMedQA/parquet/pqa_labeled/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        question = str(t['question'][i])
        ctx = t['context'][i].as_py()
        contexts = ctx.get('contexts', []) if isinstance(ctx, dict) else []
        labels = ctx.get('labels', []) if isinstance(ctx, dict) else []
        paragraphs = []
        for j, p in enumerate(contexts):
            label = labels[j] if j < len(labels) else ''
            if label:
                paragraphs.append(f'{label}: {p}')
            else:
                paragraphs.append(str(p))
        abstract = '\n\n'.join(paragraphs)
        problem = f'Abstract:\n{abstract}\n\nQuestion: {question}'
        row = {
            'id': str(t['pubid'][i]),
            'problem': problem,
            'answer': str(t['final_decision'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=pubmedqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the biomedical research abstract and answer the question. Print only one of: yes, no, or maybe. Nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
