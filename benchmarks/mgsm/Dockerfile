# MGSM (Multilingual Grade School Math)
# 2750 tasks (250 per language x 11 languages). Data: HuggingFace juletxara/mgsm
# Agent prints answer to stdout. Test compares to expected numeric answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mgsm"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="MGSM - multilingual grade school math reasoning across 11 languages"
LABEL dock.benchmark.tasks="2750"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="8b764d53dba2de1f684b836bb45c7ce389900fde"
LABEL dock.benchmark.url="https://huggingface.co/datasets/juletxara/mgsm"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mgsm"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2210.03057"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="8b764d53dba2de1f684b836bb45c7ce389900fde"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os

LANGUAGES = ['bn', 'de', 'en', 'es', 'fr', 'ja', 'ru', 'sw', 'te', 'th', 'zh']
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = f'https://huggingface.co/api/datasets/juletxara/mgsm/parquet/{lang}/test/0.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            row = {
                'id': f'{lang}_{i}',
                'problem': str(t['question'][i]),
                'answer': str(t['answer_number'][i]),
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=mgsm

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this math problem. Print only the final numeric answer as a single number. Do not include any explanation or units.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
