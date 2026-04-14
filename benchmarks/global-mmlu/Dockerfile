# Global-MMLU (CohereLabs/Global-MMLU)
# 589764 tasks (14042 test questions x 42 languages). Data: HuggingFace CohereLabs/Global-MMLU
# Agent prints letter A/B/C/D. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="global-mmlu"
LABEL dock.benchmark.description="Global-MMLU - multilingual knowledge and reasoning multiple choice across 42 languages"
LABEL dock.benchmark.tasks="589764"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="0e619dbeb34206cd48705a1a0ea7fb21cae09993"
LABEL dock.benchmark.url="https://huggingface.co/datasets/CohereLabs/Global-MMLU"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2412.03304"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os

LANGUAGES = [
    'am', 'ar', 'bn', 'cs', 'de', 'el', 'en', 'es', 'fa', 'fil',
    'fr', 'ha', 'he', 'hi', 'id', 'ig', 'it', 'ja', 'ko', 'ky',
    'lt', 'mg', 'ms', 'ne', 'nl', 'ny', 'pl', 'pt', 'ro', 'ru',
    'si', 'sn', 'so', 'sr', 'sv', 'sw', 'te', 'tr', 'uk', 'vi',
    'yo', 'zh',
]

task_idx = 0
for lang in LANGUAGES:
    url = f'https://huggingface.co/datasets/CohereLabs/Global-MMLU/resolve/0e619dbeb34206cd48705a1a0ea7fb21cae09993/{lang}/test-00000-of-00001.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        question = str(t['question'][i])
        a = str(t['option_a'][i])
        b = str(t['option_b'][i])
        c = str(t['option_c'][i])
        dopt = str(t['option_d'][i])
        answer = str(t['answer'][i]).strip().upper()
        sample_id = str(t['sample_id'][i])
        problem = f'{question}\n\nA. {a}\nB. {b}\nC. {c}\nD. {dopt}'
        open(f'{d}/id.txt', 'w').write(f'{lang}_{sample_id}')
        open(f'{d}/problem.txt', 'w').write(problem)
        open(f'{d}/answer.txt', 'w').write(answer)
        open(f'{d}/language.txt', 'w').write(lang)
        task_idx += 1
    os.remove('/tmp/data.parquet')
print(f'Wrote {task_idx} tasks')
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=global-mmlu

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
