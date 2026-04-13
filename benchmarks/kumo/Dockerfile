# Kumo (diagnostic reasoning)
# 250 tasks. Data: HuggingFace pkuHaowei/kumo-hard
# Agent observes scenario and selects actions. Test compares to ground truth.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="kumo"
LABEL dock.benchmark.description="Kumo - diagnostic reasoning"
LABEL dock.benchmark.tasks="250"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch tasks from all 5 environment splits
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
idx = 0
for env in ['ChemicalEnv', 'EducationEnv', 'FantasyEnv', 'MedicalEnv', 'MusicEnv']:
    url = f'https://huggingface.co/datasets/pkuHaowei/kumo-hard/resolve/main/data/{env}-00000-of-00001.parquet'
    urllib.request.urlretrieve(url, '/tmp/data.parquet')
    t = pq.read_table('/tmp/data.parquet')
    for i in range(len(t)):
        os.makedirs(f'/tasks/{idx}', exist_ok=True)
        open(f'/tasks/{idx}/id.txt','w').write(f'{env}_{i}')
        obs = str(t['observation'][i])
        actions = json.dumps(t['action_list'][i].as_py())
        open(f'/tasks/{idx}/problem.txt','w').write(
            f"{obs}\n\nAvailable actions:\n{actions}")
        open(f'/tasks/{idx}/answer.txt','w').write(str(t['ground_truth'][i]))
        idx += 1
    os.remove('/tmp/data.parquet')
print(f'Extracted {idx} tasks')
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=kumo

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Based on the observation, select the correct action. Print ONLY the action, nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
