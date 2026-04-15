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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="682dd723ee1e1697e00360edccf2366dc8418dd9"

RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir huggingface_hub==0.26.2 pyarrow==18.1.0

# Download dataset files (gated — requires HF_TOKEN)
ARG HF_TOKEN
RUN python3 -c "from huggingface_hub import snapshot_download; snapshot_download('gaia-benchmark/GAIA', repo_type='dataset', revision='682dd723ee1e1697e00360edccf2366dc8418dd9', allow_patterns=['2023/validation/*'], local_dir='/tmp/gaia', token='$HF_TOKEN')"

# Single-JSONL task layout (see benchmarks/RULES.md #22). Attached files are
# base64-encoded inline; entrypoint decodes them at runtime to preserve the
# agent-visible layout (attachment in /tasks/$DOCK_TASK_ID/<original_name>).
RUN python3 <<'PYEOF'
import pyarrow.parquet as pq, os, json, base64
os.makedirs('/tasks', exist_ok=True)
t = pq.read_table('/tmp/gaia/2023/validation/metadata.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['Question'][i]),
            'answer': str(t['Final answer'][i]),
        }
        fname = str(t['file_name'][i]) if t['file_name'][i] else ''
        src = f'/tmp/gaia/2023/validation/{fname}'
        if fname and os.path.exists(src):
            with open(src, 'rb') as fh:
                row['attachment_name'] = fname
                row['attachment_b64'] = base64.b64encode(fh.read()).decode('ascii')
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN rm -rf /tmp/gaia
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=gaia

# Test: exact match on agent stdout
COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

# Shared entrypoint (agent → test → results)
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

# Resolve DOCK_TASK_ID to TASK + EXPECTED_ANSWER from materialized files, then exec shared entrypoint
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -f /tasks/$DOCK_TASK_ID/attachment_b64.txt ]; then
  python3 - <<'PY'
import base64, os
tid = os.environ['DOCK_TASK_ID']
name = open(f'/tasks/{tid}/attachment_name.txt').read().strip()
data = open(f'/tasks/{tid}/attachment_b64.txt').read()
with open(f'/tasks/{tid}/{name}', 'wb') as f:
    f.write(base64.b64decode(data))
PY
  rm -f /tasks/$DOCK_TASK_ID/attachment_b64.txt /tasks/$DOCK_TASK_ID/attachment_name.txt
  chmod -R 600 /tasks/$DOCK_TASK_ID
fi
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
