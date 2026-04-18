# HellaSwag
# 10042 tasks. Data: HuggingFace Rowan/hellaswag (validation)
# Agent prints the letter of the most plausible continuation. Test compares letters.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="hellaswag"
LABEL dock.benchmark.description="HellaSwag - commonsense sentence completion: pick the most plausible ending"
LABEL dock.benchmark.tasks="10042"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="218ec52e09a7e7462a5400043bb9a69a41d06b76"
LABEL dock.benchmark.url="https://huggingface.co/datasets/Rowan/hellaswag"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1905.07830"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="218ec52e09a7e7462a5400043bb9a69a41d06b76"

# Single-JSONL task layout. Root-only; runtime entrypoint materializes the
# requested task's fields into /tasks/$DOCK_TASK_ID/ on demand.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/Rowan/hellaswag/resolve/218ec52e09a7e7462a5400043bb9a69a41d06b76/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        activity = t['activity_label'][i].as_py()
        ctx = t['ctx'][i].as_py()
        endings = t['endings'][i].as_py()
        label_idx = int(t['label'][i].as_py())
        lines = [f"Activity: {activity}", "", f"Context: {ctx}", "", "Which ending is most likely?"]
        for j, ending in enumerate(endings):
            lines.append(f"{letters[j]}. {ending}")
        row = {
            'id': str(t['ind'][i].as_py()),
            'problem': "\n".join(lines),
            'answer': letters[label_idx],
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=hellaswag

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Pick the most plausible continuation of the context. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
