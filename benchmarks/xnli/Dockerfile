# XNLI (Cross-lingual Natural Language Inference)
# 75150 tasks (5010 test examples x 15 languages). Data: HuggingFace facebook/xnli
# Agent prints entailment/neutral/contradiction. Test compares to expected label.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xnli"
LABEL eval.benchmark.description="XNLI - cross-lingual natural language inference across 15 languages"
LABEL eval.benchmark.tasks="75150"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="b8dd5d7af51114dbda02c0e3f6133f332186418e"
LABEL eval.benchmark.url="https://huggingface.co/datasets/facebook/xnli"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1809.05053"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="b8dd5d7af51114dbda02c0e3f6133f332186418e"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json

LANGUAGES = ['ar', 'bg', 'de', 'el', 'en', 'es', 'fr', 'hi', 'ru', 'sw', 'th', 'tr', 'ur', 'vi', 'zh']
LABELS = {0: 'entailment', 1: 'neutral', 2: 'contradiction'}

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = f'https://huggingface.co/datasets/facebook/xnli/resolve/b8dd5d7af51114dbda02c0e3f6133f332186418e/{lang}/test-00000-of-00001.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            premise = str(t['premise'][i])
            hypothesis = str(t['hypothesis'][i])
            label = LABELS[int(str(t['label'][i]))]
            row = {
                'id': f'{lang}_{i}',
                'problem': f'Premise: {premise}\nHypothesis: {hypothesis}',
                'answer': label,
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=xnli

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Given the premise and hypothesis, determine the relationship. Print only one word: entailment, neutral, or contradiction. Do not include any explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
