# SimpleQA (OpenAI)
# 4326 tasks. Data: HuggingFace basicv8vc/SimpleQA
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="simpleqa"
LABEL dock.benchmark.description="SimpleQA - factual question answering"
LABEL dock.benchmark.tasks="4326"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="e319282ab125c3dbd0c7fd00be2e4dd54e7e8f94"
LABEL dock.benchmark.url="https://github.com/openai/simple-evals"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/simpleqa"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/simpleqa"
LABEL dock.benchmark.paper="https://openai.com/index/introducing-simpleqa/"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve('https://huggingface.co/datasets/basicv8vc/SimpleQA/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/problem.txt','w').write(str(t['problem'][i]))
    open(f'/tasks/{i}/answer.txt','w').write(str(t['answer'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=simpleqa

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this question. Print only the answer, nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
