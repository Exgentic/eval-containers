# WMT (WMT24++)
# ~9600 tasks (~960 usable segments x 10 language pairs, en->X). Data: HuggingFace google/wmt24pp
# Agent prints translation to stdout. Test compares to expected reference translation.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="wmt"
LABEL dock.benchmark.description="WMT24++ - multilingual machine translation from English to 10 target locales"
LABEL dock.benchmark.tasks="9600"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="e65f5856b1de3319c748c15e5aec0bc2336ec3b0"
LABEL dock.benchmark.url="https://huggingface.co/datasets/google/wmt24pp"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2502.12404"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

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
task_idx = 0
for pair, lang_name in PAIRS:
    url = f'{BASE}/{pair}.jsonl'
    urllib.request.urlretrieve(url, '/tmp/data.jsonl')
    with open('/tmp/data.jsonl') as f:
        rows = [json.loads(line) for line in f]
    for r in rows:
        if str(r.get('is_bad_source', '')).lower() == 'true':
            continue
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        open(f'{d}/id.txt', 'w').write(f"{pair}_{r['segment_id']}")
        open(f'{d}/problem.txt', 'w').write(r['source'])
        open(f'{d}/answer.txt', 'w').write(r['target'])
        open(f'{d}/language.txt', 'w').write(pair)
        open(f'{d}/language_name.txt', 'w').write(lang_name)
        task_idx += 1
    os.remove('/tmp/data.jsonl')

print(f'Wrote {task_idx} tasks')
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=wmt

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  LANG_NAME=$(cat /tasks/$DOCK_TASK_ID/language_name.txt)
  export TASK="Translate the following English text into ${LANG_NAME}. Print only the translated text with no explanation, quotes, or additional commentary.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
