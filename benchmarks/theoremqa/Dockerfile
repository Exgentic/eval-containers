# TheoremQA (theorem-driven question answering)
# 800 tasks. Data: HuggingFace TIGER-Lab/TheoremQA
# Agent prints answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="theoremqa"
LABEL dock.benchmark.description="TheoremQA - STEM problems requiring the application of named theorems"
LABEL dock.benchmark.tasks="800"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="a340b1782960a712843aae3ed25f1e013cc008a5"
LABEL dock.benchmark.url="https://huggingface.co/datasets/TIGER-Lab/TheoremQA"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2305.12524"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/TIGER-Lab/TheoremQA/parquet/default/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['Question'][i]))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['Answer'][i]))
    open(f'/tasks/{i}/answer_type.txt', 'w').write(str(t['Answer_type'][i]))
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=theoremqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this theorem-based problem. Print only the final answer (a number, expression, or value). Do not include any explanation or units.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
