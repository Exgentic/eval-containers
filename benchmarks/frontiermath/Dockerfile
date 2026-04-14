# FrontierMath (Epoch AI)
# Research-level math problems. Dataset is gated/private; only the public
# sample set is available on HuggingFace. Agent prints final answer to stdout.
#
# TODO: FrontierMath full set (~300 problems) is held privately by Epoch AI and
# is not publicly downloadable. Using the public sample split as a placeholder.
# Replace with the full set once access is granted (set HF_TOKEN and data_revision).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="frontiermath"
LABEL dock.benchmark.description="FrontierMath - research-level math problems (Epoch AI)"
LABEL dock.benchmark.tasks="10"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://epochai.org/frontiermath"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2411.04872"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow huggingface_hub

# Public sample set. The full evaluation set is gated and held by Epoch AI.
ARG HF_TOKEN=""
RUN python3 <<'PYEOF'
import os, json
from huggingface_hub import snapshot_download
try:
    snapshot_download(
        'epoch-ai/frontiermath',
        repo_type='dataset',
        local_dir='/tmp/fm',
        token=os.environ.get('HF_TOKEN') or None,
    )
except Exception as e:
    # TODO: dataset is gated — placeholder task written when unavailable
    os.makedirs('/tmp/fm', exist_ok=True)
    print(f'Could not fetch FrontierMath ({e}); writing placeholder task')

# Try parquet first, fall back to JSONL, fall back to placeholder
import glob
rows = []
for pq_path in glob.glob('/tmp/fm/**/*.parquet', recursive=True):
    import pyarrow.parquet as pq
    t = pq.read_table(pq_path)
    cols = t.column_names
    q_col = next((c for c in cols if c.lower() in ('problem', 'question', 'statement')), cols[0])
    a_col = next((c for c in cols if c.lower() in ('answer', 'final_answer', 'solution')), cols[-1])
    id_col = next((c for c in cols if c.lower() in ('id', 'problem_id')), None)
    for i in range(len(t)):
        rows.append({
            'id': str(t[id_col][i]) if id_col else str(i),
            'problem': str(t[q_col][i]),
            'answer': str(t[a_col][i]),
        })
    break

if not rows:
    # TODO: placeholder when dataset is not accessible
    for i in range(10):
        rows.append({
            'id': f'placeholder-{i}',
            'problem': f'TODO: FrontierMath problem {i}. Replace by setting HF_TOKEN with Epoch AI access.',
            'answer': 'TODO',
        })

os.makedirs('/tasks', exist_ok=True)
for i, r in enumerate(rows):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(r['id'])
    open(f'/tasks/{i}/problem.txt', 'w').write(r['problem'])
    open(f'/tasks/{i}/answer.txt', 'w').write(r['answer'])
PYEOF
RUN rm -rf /tmp/fm && pip uninstall -y pyarrow huggingface_hub
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=frontiermath

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this research-level mathematics problem. Print only the final answer on the last line.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
