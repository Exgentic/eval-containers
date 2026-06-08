# AppWorld
# 732 tasks across 9 simulated apps. Data: HuggingFace LukaszTP/AppWorld-Tasks
# Upstream image: ghcr.io/stonybrooknlp/appworld — includes all APIs in-process.
# Evaluation is state-based (database checks), requires the AppWorld runtime.
# Externally graded: test writes -1.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ghcr.io/stonybrooknlp/appworld:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="appworld"
LABEL eval.benchmark.upstream_base="ghcr.io/stonybrooknlp/appworld:latest"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="AppWorld - 9 simulated apps with 457 APIs"
LABEL eval.benchmark.tasks="732"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="refs/convert/parquet"
LABEL eval.benchmark.url="https://github.com/stonybrooknlp/appworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2407.18901"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

ENV EVAL_BENCHMARK=appworld

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
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
