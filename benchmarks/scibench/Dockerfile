# SciBench (college-level scientific problems)
# 692 tasks. Data: HuggingFace xw27/scibench
# Agent prints answer to stdout. Test compares to expected numeric answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="scibench"
LABEL dock.benchmark.description="SciBench - college-level scientific problems in math, physics, and chemistry"
LABEL dock.benchmark.tasks="692"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="93931252bc1b71d495e67390235940643d926958"
LABEL dock.benchmark.url="https://github.com/mandyyyyii/scibench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2307.10635"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/xw27/scibench/parquet/default/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['problemid'][i]).strip())
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['problem_text'][i]))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answer_number'][i]).strip())
    open(f'/tasks/{i}/unit.txt', 'w').write(str(t['unit'][i]))
    open(f'/tasks/{i}/source.txt', 'w').write(str(t['source'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=scibench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  UNIT=$(cat /tasks/$DOCK_TASK_ID/unit.txt 2>/dev/null || echo "")
  export TASK="Solve this college-level science problem. Print only the final numeric answer as a single number (no units, no explanation). Expected unit: ${UNIT}.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
