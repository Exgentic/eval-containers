# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="90"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="13f9e12f613e720c2a2b2f345dd04b998a29494d"
LABEL dock.benchmark.url="https://huggingface.co/datasets/AI-MO/aimo-validation-aime"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/aime"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/aime2024"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="13f9e12f613e720c2a2b2f345dd04b998a29494d"

RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve('https://huggingface.co/datasets/AI-MO/aimo-validation-aime/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['id'][i]),
            'problem': str(t['problem'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(t)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=aime

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this problem. Print only the answer as a single integer.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
