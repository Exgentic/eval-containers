# XNLI (Cross-lingual Natural Language Inference)
# 75150 tasks (5010 test examples x 15 languages). Data: HuggingFace facebook/xnli
# Agent prints entailment/neutral/contradiction. Test compares to expected label.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="xnli"
LABEL dock.benchmark.description="XNLI - cross-lingual natural language inference across 15 languages"
LABEL dock.benchmark.tasks="75150"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="b8dd5d7af51114dbda02c0e3f6133f332186418e"
LABEL dock.benchmark.url="https://huggingface.co/datasets/facebook/xnli"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1809.05053"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os

LANGUAGES = ['ar', 'bg', 'de', 'el', 'en', 'es', 'fr', 'hi', 'ru', 'sw', 'th', 'tr', 'ur', 'vi', 'zh']
LABELS = {0: 'entailment', 1: 'neutral', 2: 'contradiction'}

task_idx = 0
for lang in LANGUAGES:
    url = f'https://huggingface.co/datasets/facebook/xnli/resolve/b8dd5d7af51114dbda02c0e3f6133f332186418e/{lang}/test-00000-of-00001.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        open(f'{d}/id.txt', 'w').write(f'{lang}_{i}')
        premise = str(t['premise'][i])
        hypothesis = str(t['hypothesis'][i])
        label = LABELS[int(str(t['label'][i]))]
        problem = f'Premise: {premise}\nHypothesis: {hypothesis}'
        open(f'{d}/problem.txt', 'w').write(problem)
        open(f'{d}/answer.txt', 'w').write(label)
        open(f'{d}/language.txt', 'w').write(lang)
        task_idx += 1
    os.remove('/tmp/data.parquet')
print(f'Wrote {task_idx} tasks')
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=xnli

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Given the premise and hypothesis, determine the relationship. Print only one word: entailment, neutral, or contradiction. Do not include any explanation.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
