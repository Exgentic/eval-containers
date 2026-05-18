# MRCR (Multi-Round Coreference Resolution) v2 (OpenAI)
# 2400 tasks. Data: HuggingFace openai/mrcr
# Long-context retrieval: agent must find the i-th instance of a repeated
# request hidden in a synthetic multi-turn conversation.
# Scoring: difflib.SequenceMatcher ratio with required prefix check.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mrcr"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MRCR v2 - Multi-Round Coreference Resolution (long-context retrieval)"
LABEL eval.benchmark.tasks="2400"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="f4c69fae7cf81f7ca26b9fee34b392a50f6b8a1d"
LABEL eval.benchmark.url="https://huggingface.co/datasets/openai/mrcr"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2505.14993"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="f4c69fae7cf81f7ca26b9fee34b392a50f6b8a1d"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/. Pre-render prompt.json into problem.txt
# (flattened conversation text) so the entrypoint stays simple.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, pyarrow as pa, json, os

tables = []
for shard in range(6):
    url = (
        'https://huggingface.co/datasets/openai/mrcr/resolve/'
        f'refs%2Fconvert%2Fparquet/default/train/{shard:04d}.parquet'
    )
    path = f'/tmp/shard{shard}.parquet'
    urllib.request.urlretrieve(url, path)
    tables.append(pq.read_table(path))
    os.remove(path)

t = pa.concat_tables(tables)
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        answer = t['answer'][i].as_py()
        prefix = t['random_string_to_prepend'][i].as_py()
        messages = json.loads(t['prompt'][i].as_py())
        lines = []
        for m in messages:
            role = m['role'].upper()
            lines.append(f'{role}: {m["content"]}')
        lines.append('')
        lines.append(
            'IMPORTANT: You MUST start your response with the following '
            'exact string (no extra whitespace or characters before it):'
        )
        lines.append(prefix)
        lines.append('')
        lines.append('Then provide the requested content immediately after that prefix.')
        row = {
            'id': str(i),
            'problem': '\n'.join(lines),
            'answer': answer,
            'prefix': prefix,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mrcr

# Custom test: SequenceMatcher ratio with prefix check (per MRCR spec)
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 <<'GRADE'
from difflib import SequenceMatcher
import os

task_id = os.environ.get('EVAL_TASK_ID', '')

try:
    response = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    response = ''

answer = open(f'/tasks/{task_id}/answer.txt').read()
prefix = open(f'/tasks/{task_id}/prefix.txt').read()

if not response.startswith(prefix):
    reward = 0.0
else:
    r = response.removeprefix(prefix)
    a = answer.removeprefix(prefix)
    reward = SequenceMatcher(None, r, a).ratio()

open('/logs/verifier/reward.txt', 'w').write(f'{reward:.6f}\n')
GRADE
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  cp /tasks/$EVAL_TASK_ID/problem.txt /app/task.txt
  chmod 644 /app/task.txt
  export TASK="Your task is described in the file /app/task.txt — read it and follow the instructions inside."
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
