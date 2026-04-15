# GAIA (General AI Assistants)
# 165 tasks. Data: HuggingFace gaia-benchmark/GAIA (gated, requires HF_TOKEN at build time)
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gaia"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="GAIA - General AI Assistants"
LABEL dock.benchmark.tasks="165"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"
LABEL dock.benchmark.data_revision="682dd723ee1e1697e00360edccf2366dc8418dd9"
LABEL dock.benchmark.url="https://github.com/gaia-benchmark/GAIA"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/gaia"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/gaia"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2311.12983"

RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir huggingface_hub

# Download dataset files (gated — requires HF_TOKEN)
ARG HF_TOKEN
RUN python3 -c "from huggingface_hub import snapshot_download; snapshot_download('gaia-benchmark/GAIA', repo_type='dataset', revision='682dd723ee1e1697e00360edccf2366dc8418dd9', allow_patterns=['2023/validation/*'], local_dir='/tmp/gaia', token='$HF_TOKEN')"

# Extract tasks to flat files from the parquet
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import pyarrow.parquet as pq, os, shutil
t = pq.read_table('/tmp/gaia/2023/validation/metadata.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    with open(f'/tasks/{i}/id.txt', 'w') as f:
        f.write(str(t['task_id'][i]))
    with open(f'/tasks/{i}/problem.txt', 'w') as f:
        f.write(str(t['Question'][i]))
    with open(f'/tasks/{i}/answer.txt', 'w') as f:
        f.write(str(t['Final answer'][i]))
    fname = str(t['file_name'][i])
    src = f'/tmp/gaia/2023/validation/{fname}'
    if fname and os.path.exists(src):
        shutil.move(src, f'/tasks/{i}/{fname}')
PYEOF
RUN rm -rf /tmp/gaia && pip uninstall -y pyarrow huggingface_hub
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=gaia

# Test: exact match on agent stdout
COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

# Shared entrypoint (agent → test → results)
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

# Resolve DOCK_TASK_ID to TASK + EXPECTED_ANSWER from flat files, then exec shared entrypoint
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this question. Print only the answer, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
  # Copy attached files to /app where agent can read them
  for f in /tasks/$DOCK_TASK_ID/*; do
    name=$(basename "$f")
    case "$name" in problem.txt|answer.txt|id.txt) continue;; esac
    cp "$f" /app/
  done
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
