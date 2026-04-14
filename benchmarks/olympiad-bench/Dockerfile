# OlympiadBench (text-only English math + physics)
# 910 tasks. Data: HuggingFace Hothan/OlympiadBench (OE_TO_*_en_COMP configs)
# Agent prints answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="olympiad-bench"
LABEL dock.benchmark.description="OlympiadBench - olympiad-level math and physics problems (text-only English split)"
LABEL dock.benchmark.tasks="910"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="91184b52131e7fc9455fef848035173aea8cc01a"
LABEL dock.benchmark.url="https://huggingface.co/datasets/Hothan/OlympiadBench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2402.14008"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, ast

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

task_idx = 0
for cfg in CONFIGS:
    url = f'https://huggingface.co/api/datasets/Hothan/OlympiadBench/parquet/{cfg}/train/0.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        os.makedirs(f'/tasks/{task_idx}', exist_ok=True)
        open(f'/tasks/{task_idx}/id.txt', 'w').write(f"{cfg}_{t['id'][i]}")
        open(f'/tasks/{task_idx}/problem.txt', 'w').write(str(t['question'][i]))
        open(f'/tasks/{task_idx}/answer.txt', 'w').write(parse_final_answer(t['final_answer'][i]))
        open(f'/tasks/{task_idx}/subject.txt', 'w').write(str(t['subject'][i]))
        task_idx += 1
    os.remove('/tmp/data.parquet')
print('total tasks:', task_idx)
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=olympiad-bench

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this olympiad problem. Print only the final answer in its simplest form (a number, expression, or semicolon-separated list for multi-part answers). Do not include any explanation or units.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
