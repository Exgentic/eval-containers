# AppWorld
# 732 tasks across 9 simulated apps. Data: HuggingFace LukaszTP/AppWorld-Tasks
# Upstream image: ghcr.io/stonybrooknlp/appworld — includes all APIs in-process.
# Evaluation is state-based (database checks), requires the AppWorld runtime.
# Externally graded: test writes -1.

FROM ghcr.io/stonybrooknlp/appworld:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="appworld"
LABEL dock.benchmark.upstream_base="ghcr.io/stonybrooknlp/appworld:latest"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="AppWorld - 9 simulated apps with 457 APIs"
LABEL dock.benchmark.tasks="732"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="refs/convert/parquet"
LABEL dock.benchmark.url="https://github.com/stonybrooknlp/appworld"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2407.18901"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="refs/convert/parquet"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

ENV DOCK_BENCHMARK=appworld

# Fetch task metadata from HuggingFace and write to a single JSONL file.
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/LukaszTP/AppWorld-Tasks/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['instruction'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app

# Test: AppWorld evaluation is state-based (database checks) and requires
# the full AppWorld runtime. This is externally graded — always write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
