# Kumo (diagnostic reasoning)
# 250 tasks. Data: HuggingFace pkuHaowei/kumo-hard
# Agent observes scenario and selects actions. Test compares to ground truth.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="kumo"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="Kumo - diagnostic reasoning"
LABEL dock.benchmark.tasks="250"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/Haowei-PKU/Kumo"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/kumo"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2505.08857"
ENV DOCK_BENCHMARK_VERSION_DEFAULT=""

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for env in ['ChemicalEnv', 'EducationEnv', 'FantasyEnv', 'MedicalEnv', 'MusicEnv']:
        url = f'https://huggingface.co/datasets/pkuHaowei/kumo-hard/resolve/main/data/{env}-00000-of-00001.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            obs = str(t['observation'][i])
            actions = json.dumps(t['action_list'][i].as_py())
            row = {
                'id': f'{env}_{i}',
                'problem': f"{obs}\n\nAvailable actions:\n{actions}",
                'answer': str(t['ground_truth'][i]),
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Extracted {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=kumo

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Based on the observation, select the correct action. Print ONLY the action, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
