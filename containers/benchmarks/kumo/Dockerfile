# Kumo (diagnostic reasoning)
# 250 tasks. Data: HuggingFace pkuHaowei/kumo-hard
# Agent observes scenario and selects actions. Test compares to ground truth.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="kumo"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Kumo - diagnostic reasoning"
LABEL eval.benchmark.tasks="250"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/Haowei-PKU/Kumo"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/kumo"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2505.08857"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
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
ENV EVAL_BENCHMARK=kumo

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Based on the observation, select the correct action. Print ONLY the action, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
