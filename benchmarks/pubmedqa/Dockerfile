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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/qiaojin/PubMedQA/parquet/pqa_labeled/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['pubid'][i]))
    question = str(t['question'][i])
    ctx = t['context'][i].as_py()
    # ctx is a struct with 'contexts' (list of paragraphs) and 'labels' (list of section headers)
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
    open(f'/tasks/{i}/problem.txt', 'w').write(problem)
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['final_decision'][i]))
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=pubmedqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the biomedical research abstract and answer the question. Print only one of: yes, no, or maybe. Nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
