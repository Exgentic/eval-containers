# AssistantBench (Tel Aviv NLP)
# 33 validation tasks. Data: HuggingFace AssistantBench/AssistantBench
# Open-domain, time-consuming web research tasks. Agent browses the web and
# prints a short answer. Test compares to the reference answer (exact match).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="assistantbench"
LABEL dock.benchmark.description="AssistantBench - realistic long-horizon web research"
LABEL dock.benchmark.tasks="33"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"
LABEL dock.benchmark.data_revision="482cbbc0400f6d048438c4021727f21a10cbff49"
LABEL dock.benchmark.url="https://github.com/oriyor/assistant-bench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2407.15711"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/AssistantBench/AssistantBench/parquet/default/validation/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt','w').write(str(t['id'][i]))
    open(f'/tasks/{i}/problem.txt','w').write(str(t['task'][i]))
    open(f'/tasks/{i}/answer.txt','w').write(str(t['answer'][i]))
print(f'Extracted {len(t)} tasks')
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=assistantbench

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Browse the web to answer this question. Print only the answer, nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
