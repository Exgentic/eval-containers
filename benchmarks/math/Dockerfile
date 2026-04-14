# MATH (Hendrycks competition mathematics)
# 5000 tasks (7 subjects x test split). Data: HuggingFace EleutherAI/hendrycks_math
# Agent prints answer to stdout. Test compares to boxed expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="math"
LABEL dock.benchmark.description="MATH - competition mathematics problems across 7 subjects (Hendrycks et al.)"
LABEL dock.benchmark.tasks="5000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="21a5633873b6a120296cce3e2df9d5550074f4a3"
LABEL dock.benchmark.url="https://huggingface.co/datasets/EleutherAI/hendrycks_math"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2103.03874"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, re

SUBJECTS = ['algebra', 'counting_and_probability', 'geometry',
            'intermediate_algebra', 'number_theory', 'prealgebra', 'precalculus']

def extract_boxed(s):
    idx = s.rfind('\\boxed{')
    if idx < 0:
        return s.strip()
    i = idx + len('\\boxed{')
    depth = 1
    out = []
    while i < len(s) and depth > 0:
        c = s[i]
        if c == '{':
            depth += 1
            out.append(c)
        elif c == '}':
            depth -= 1
            if depth == 0:
                break
            out.append(c)
        else:
            out.append(c)
        i += 1
    return ''.join(out).strip()

task_idx = 0
for subject in SUBJECTS:
    url = f'https://huggingface.co/api/datasets/EleutherAI/hendrycks_math/parquet/{subject}/test/0.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        os.makedirs(f'/tasks/{task_idx}', exist_ok=True)
        open(f'/tasks/{task_idx}/id.txt', 'w').write(f'{subject}_{i}')
        open(f'/tasks/{task_idx}/problem.txt', 'w').write(str(t['problem'][i]))
        open(f'/tasks/{task_idx}/answer.txt', 'w').write(extract_boxed(str(t['solution'][i])))
        open(f'/tasks/{task_idx}/subject.txt', 'w').write(subject)
        task_idx += 1
    os.remove('/tmp/data.parquet')
print('total tasks:', task_idx)
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=math

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this math problem. Print only the final answer in its simplest form (a number, fraction, or LaTeX expression as it would appear inside \\boxed{}). Do not include any explanation.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
