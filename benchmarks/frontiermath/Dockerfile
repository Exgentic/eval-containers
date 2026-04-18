# FrontierMath (Epoch AI)
# Research-level math problems. Dataset is gated/private; only the public
# sample set is available on HuggingFace. Agent prints final answer to stdout.
#
# FUTURE: FrontierMath full set (~300 problems) is held privately by Epoch AI and
# is not publicly downloadable. Using the public sample split as a placeholder.
# Replace with the full set once access is granted (set HF_TOKEN and data_revision).

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="frontiermath"
LABEL dock.benchmark.description="FrontierMath - research-level math problems (Epoch AI)"
LABEL dock.benchmark.tasks="10"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://epochai.org/frontiermath"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2411.04872"
ENV DOCK_BENCHMARK_VERSION_DEFAULT=""

RUN pip install --no-cache-dir pyarrow==18.1.0 huggingface_hub==0.26.2

# Public sample set. The full evaluation set is gated and held by Epoch AI.
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN 2>/dev/null || echo "$HF_TOKEN") && \
    export HF_TOKEN && \
    python3 <<'PYEOF'
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
    os.makedirs('/tmp/fm', exist_ok=True)
    print(f'Could not fetch FrontierMath ({e}); writing placeholder task')

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
    # FrontierMath is gated by Epoch AI. We intentionally fail the
    # build rather than ship placeholder tasks that would silently
    # grade agents against garbage. To unlock, set HF_TOKEN to an
    # account with access and rebuild.
    raise SystemExit(
        'frontiermath: upstream dataset not accessible. '
        'Set HF_TOKEN with Epoch AI access and rebuild. '
        'Refusing to ship placeholder task content.'
    )

os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for r in rows:
        out.write(json.dumps(r) + '\n')
print(f'Wrote {len(rows)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=frontiermath

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this research-level mathematics problem. Print only the final answer on the last line.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
