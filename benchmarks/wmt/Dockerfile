# WMT (WMT24++)
# ~9600 tasks (~960 usable segments x 10 language pairs, en->X). Data: HuggingFace google/wmt24pp
# Agent prints translation to stdout. Test compares to expected reference translation.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="wmt"
LABEL eval.benchmark.description="WMT24++ - multilingual machine translation from English to 10 target locales"
LABEL eval.benchmark.tasks="9600"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="e65f5856b1de3319c748c15e5aec0bc2336ec3b0"
LABEL eval.benchmark.url="https://huggingface.co/datasets/google/wmt24pp"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.12404"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="e65f5856b1de3319c748c15e5aec0bc2336ec3b0"

RUN python3 <<'PYEOF'
import urllib.request, json, os

# 10 language pairs from WMT24++ (en -> X)
PAIRS = [
    ('en-fr_FR', 'French (France)'),
    ('en-de_DE', 'German (Germany)'),
    ('en-es_MX', 'Spanish (Mexico)'),
    ('en-ru_RU', 'Russian (Russia)'),
    ('en-zh_CN', 'Simplified Chinese (Mainland)'),
    ('en-ja_JP', 'Japanese (Japan)'),
    ('en-ar_EG', 'Arabic (Egypt)'),
    ('en-hi_IN', 'Hindi (India)'),
    ('en-pt_BR', 'Portuguese (Brazil)'),
    ('en-ko_KR', 'Korean (South Korea)'),
]

BASE = 'https://huggingface.co/datasets/google/wmt24pp/resolve/e65f5856b1de3319c748c15e5aec0bc2336ec3b0'
os.makedirs('/tasks', exist_ok=True)
task_idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for pair, lang_name in PAIRS:
        url = f'{BASE}/{pair}.jsonl'
        urllib.request.urlretrieve(url, '/tmp/data.jsonl')
        with open('/tmp/data.jsonl') as f:
            rows = [json.loads(line) for line in f]
        for r in rows:
            if str(r.get('is_bad_source', '')).lower() == 'true':
                continue
            row = {
                'id': f"{pair}_{r['segment_id']}",
                'problem': r['source'],
                'answer': r['target'],
                'language': pair,
                'language_name': lang_name,
            }
            out.write(json.dumps(row) + '\n')
            task_idx += 1
        os.remove('/tmp/data.jsonl')

print(f'Wrote {task_idx} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=wmt

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  LANG_NAME=$(cat /tasks/$EVAL_TASK_ID/language_name.txt)
  export TASK="Translate the following English text into ${LANG_NAME}. Print only the translated text with no explanation, quotes, or additional commentary.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
