# OlympiadBench (text-only English math + physics)
# 910 tasks. Data: HuggingFace Hothan/OlympiadBench (OE_TO_*_en_COMP configs)
# Agent prints answer to stdout. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="olympiad-bench"
LABEL dock.benchmark.description="OlympiadBench - olympiad-level math and physics problems (text-only English split)"
LABEL dock.benchmark.tasks="910"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="91184b52131e7fc9455fef848035173aea8cc01a"
LABEL dock.benchmark.url="https://huggingface.co/datasets/Hothan/OlympiadBench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2402.14008"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="91184b52131e7fc9455fef848035173aea8cc01a"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, ast

CONFIGS = ['OE_TO_maths_en_COMP', 'OE_TO_physics_en_COMP']

def parse_final_answer(raw):
    s = str(raw).strip()
    try:
        val = ast.literal_eval(s)
        if isinstance(val, (list, tuple)):
            return ' ; '.join(str(x).strip() for x in val)
        return str(val).strip()
    except Exception:
        return s

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for cfg in CONFIGS:
        url = f'https://huggingface.co/api/datasets/Hothan/OlympiadBench/parquet/{cfg}/train/0.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            row = {
                'id': f"{cfg}_{t['id'][i]}",
                'problem': str(t['question'][i]),
                'answer': parse_final_answer(t['final_answer'][i]),
                'subject': str(t['subject'][i]),
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print('total tasks:', count)
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=olympiad-bench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this olympiad problem. Print only the final answer in its simplest form (a number, expression, or semicolon-separated list for multi-part answers). Do not include any explanation or units.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
