# GSM8K (Grade School Math 8K)
# 1319 tasks. Data: HuggingFace openai/gsm8k (main/test)
# Agent prints answer to stdout. Test compares to expected numeric answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gsm8k"
LABEL dock.benchmark.description="GSM8K - grade school math word problems requiring multi-step reasoning"
LABEL dock.benchmark.tasks="1319"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="740312add88f781978c0658806c59bc2815b9866"
LABEL dock.benchmark.url="https://huggingface.co/datasets/openai/gsm8k"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2110.14168"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/openai/gsm8k/parquet/main/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['question'][i]))
    full_answer = str(t['answer'][i])
    numeric = full_answer.split('####')[-1].strip().replace(',', '')
    open(f'/tasks/{i}/answer.txt', 'w').write(numeric)
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=gsm8k

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this grade school math problem. Print only the final numeric answer as a single number with no units, commas, or explanation.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
