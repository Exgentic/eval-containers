# WinoGrande (winogrande_xl validation)
# 1267 tasks. Data: HuggingFace allenai/winogrande (winogrande_xl/validation)
# Agent prints "1" or "2" indicating which option fills the blank.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="winogrande"
LABEL dock.benchmark.description="WinoGrande - large-scale Winograd-style commonsense pronoun resolution (xl validation)"
LABEL dock.benchmark.tasks="1267"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="01e74176c63542e6b0bcb004dcdea22d94fb67b5"
LABEL dock.benchmark.url="https://huggingface.co/datasets/allenai/winogrande"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1907.10641"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/winogrande/resolve/01e74176c63542e6b0bcb004dcdea22d94fb67b5/winogrande_xl/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    sentence = t['sentence'][i].as_py()
    o1 = t['option1'][i].as_py()
    o2 = t['option2'][i].as_py()
    answer = str(t['answer'][i].as_py())
    problem = f"{sentence}\n\n1. {o1}\n2. {o2}"
    open(f'/tasks/{i}/problem.txt', 'w').write(problem)
    open(f'/tasks/{i}/answer.txt', 'w').write(answer)
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=winogrande

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="The sentence has a blank marked with _. Choose which option correctly fills the blank. Print only the digit 1 or 2, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
