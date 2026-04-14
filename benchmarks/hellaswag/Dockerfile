# HellaSwag
# 10042 tasks. Data: HuggingFace Rowan/hellaswag (validation)
# Agent prints the letter of the most plausible continuation. Test compares letters.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="hellaswag"
LABEL dock.benchmark.description="HellaSwag - commonsense sentence completion: pick the most plausible ending"
LABEL dock.benchmark.tasks="10042"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="218ec52e09a7e7462a5400043bb9a69a41d06b76"
LABEL dock.benchmark.url="https://huggingface.co/datasets/Rowan/hellaswag"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1905.07830"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/Rowan/hellaswag/resolve/218ec52e09a7e7462a5400043bb9a69a41d06b76/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['ind'][i].as_py()))
    activity = t['activity_label'][i].as_py()
    ctx = t['ctx'][i].as_py()
    endings = t['endings'][i].as_py()
    label_idx = int(t['label'][i].as_py())
    lines = [f"Activity: {activity}", "", f"Context: {ctx}", "", "Which ending is most likely?"]
    for j, ending in enumerate(endings):
        lines.append(f"{letters[j]}. {ending}")
    open(f'/tasks/{i}/problem.txt', 'w').write("\n".join(lines))
    open(f'/tasks/{i}/answer.txt', 'w').write(letters[label_idx])
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=hellaswag

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Pick the most plausible continuation of the context. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
