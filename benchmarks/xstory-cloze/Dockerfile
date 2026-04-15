# XStoryCloze (Cross-lingual Story Cloze)
# 16621 tasks (1511 eval examples x 11 languages). Data: HuggingFace juletxara/xstory_cloze
# Agent prints 1 or 2. Test compares to expected correct ending.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="xstory-cloze"
LABEL dock.benchmark.description="XStoryCloze - cross-lingual commonsense story completion across 11 languages"
LABEL dock.benchmark.tasks="16621"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c4c2d88a1ec8b37fe22166d2a610f272726724b6"
LABEL dock.benchmark.url="https://huggingface.co/datasets/juletxara/xstory_cloze"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2112.10668"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os

LANGUAGES = ['ar', 'en', 'es', 'eu', 'hi', 'id', 'my', 'ru', 'sw', 'te', 'zh']

task_idx = 0
for lang in LANGUAGES:
    url = f'https://huggingface.co/datasets/juletxara/xstory_cloze/resolve/refs%2Fconvert%2Fparquet/{lang}/eval/0000.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        s1 = str(t['input_sentence_1'][i])
        s2 = str(t['input_sentence_2'][i])
        s3 = str(t['input_sentence_3'][i])
        s4 = str(t['input_sentence_4'][i])
        q1 = str(t['sentence_quiz1'][i])
        q2 = str(t['sentence_quiz2'][i])
        answer = str(t['answer_right_ending'][i])
        problem = f'{s1}\n{s2}\n{s3}\n{s4}\n\nWhich ending is correct?\n1. {q1}\n2. {q2}'
        open(f'{d}/id.txt', 'w').write(f'{lang}_{i}')
        open(f'{d}/problem.txt', 'w').write(problem)
        open(f'{d}/answer.txt', 'w').write(answer)
        open(f'{d}/language.txt', 'w').write(lang)
        task_idx += 1
    os.remove('/tmp/data.parquet')
print(f'Wrote {task_idx} tasks')
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=xstory-cloze

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the story and choose the correct ending. Print only the number 1 or 2, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
