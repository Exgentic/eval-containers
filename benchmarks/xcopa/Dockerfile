# XCOPA (Cross-lingual Choice of Plausible Alternatives)
# 5500 tasks (500 test examples x 11 languages). Data: HuggingFace cambridgeltl/xcopa
# Agent prints A or B. Test compares to expected choice.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="xcopa"
LABEL dock.benchmark.description="XCOPA - cross-lingual causal commonsense reasoning across 11 languages"
LABEL dock.benchmark.tasks="5500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="042f78955ba48e6404616762fa6e05e839c3907a"
LABEL dock.benchmark.url="https://huggingface.co/datasets/cambridgeltl/xcopa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2005.00333"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os

LANGUAGES = ['et', 'ht', 'id', 'it', 'qu', 'sw', 'ta', 'th', 'tr', 'vi', 'zh']

task_idx = 0
for lang in LANGUAGES:
    url = f'https://huggingface.co/datasets/cambridgeltl/xcopa/resolve/042f78955ba48e6404616762fa6e05e839c3907a/{lang}/test-00000-of-00001.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        premise = str(t['premise'][i])
        choice1 = str(t['choice1'][i])
        choice2 = str(t['choice2'][i])
        question = str(t['question'][i])
        label = int(str(t['label'][i]))
        if question == 'cause':
            prompt_q = 'What was the cause?'
        else:
            prompt_q = 'What was the effect?'
        problem = f'{premise}\n{prompt_q}\nA. {choice1}\nB. {choice2}'
        open(f'{d}/id.txt', 'w').write(f'{lang}_{i}')
        open(f'{d}/problem.txt', 'w').write(problem)
        open(f'{d}/answer.txt', 'w').write('A' if label == 0 else 'B')
        open(f'{d}/language.txt', 'w').write(lang)
        task_idx += 1
    os.remove('/tmp/data.parquet')
print(f'Wrote {task_idx} tasks')
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=xcopa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Choose the more plausible alternative. Print only the letter A or B, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
