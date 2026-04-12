# syntax=docker/dockerfile:1
# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="90"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN pip install --no-cache-dir pyarrow

# Download and convert tasks from HuggingFace parquet
RUN <<'EOF'
python3 -c "import urllib.request; urllib.request.urlretrieve('https://huggingface.co/datasets/AI-MO/aimo-validation-aime/resolve/main/data/train-00000-of-00001.parquet', '/tmp/aime.parquet')"
python3 -c "
import json, pyarrow.parquet as pq, os
t = pq.read_table('/tmp/aime.parquet')
tasks = {}
for i in range(len(t)):
    tasks[str(t['id'][i])] = {
        'instruction': 'Solve the following problem. Write your final answer as a single integer to /app/answer.txt.\n\n' + str(t['problem'][i]),
        'expected_answer': str(t['answer'][i]),
    }
os.makedirs('/tasks', exist_ok=True)
json.dump(tasks, open('/tasks/tasks.json', 'w'))
print(f'{len(tasks)} tasks')
"
chmod 600 /tasks/tasks.json
rm /tmp/aime.parquet
EOF

RUN useradd -m -s /bin/bash agent

WORKDIR /app
ENV BENCHMARK=aime

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
