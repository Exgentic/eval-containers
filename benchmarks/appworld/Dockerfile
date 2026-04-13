# syntax=docker/dockerfile:1
# AppWorld
# 732 tasks across 9 simulated apps. Data: HuggingFace LukaszTP/AppWorld-Tasks
# Upstream image: ghcr.io/stonybrooknlp/appworld — includes all APIs in-process.
# Evaluation requires the AppWorld runtime (in upstream image).

FROM ghcr.io/stonybrooknlp/appworld:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="appworld"
LABEL dock.benchmark.description="AppWorld - 9 simulated apps with 457 APIs"
LABEL dock.benchmark.tasks="732"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

ENV BENCHMARK=appworld

# Fetch task metadata from HuggingFace
RUN pip install --no-cache-dir pyarrow 2>/dev/null || true
RUN python3 <<'PYEOF'
import os, json
try:
    import urllib.request, pyarrow.parquet as pq
    urllib.request.urlretrieve(
        'https://huggingface.co/datasets/LukaszTP/AppWorld-Tasks/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet',
        '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        os.makedirs(f'/tasks/{i}', exist_ok=True)
        open(f'/tasks/{i}/id.txt','w').write(str(t['task_id'][i]))
        open(f'/tasks/{i}/problem.txt','w').write(str(t['instruction'][i]))
    print(f'Extracted {len(t)} tasks')
except Exception as e:
    print(f'Warning: could not fetch tasks: {e}')
PYEOF
RUN rm -f /tmp/data.parquet
RUN pip uninstall -y pyarrow 2>/dev/null || true
RUN chmod -R 600 /tasks 2>/dev/null || true

# Test: AppWorld evaluation requires the upstream runtime.
# Placeholder — checks if agent produced output.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
if [ -s /output/agent/stdout.log ]; then
  echo -1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
