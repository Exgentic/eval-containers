# GSM8K (Grade School Math 8K)
# 1319 tasks. Data: HuggingFace openai/gsm8k (main/test)
# Agent prints answer to stdout. Test compares to expected numeric answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="gsm8k"
LABEL eval.benchmark.description="GSM8K - grade school math word problems requiring multi-step reasoning"
LABEL eval.benchmark.tasks="1319"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="740312add88f781978c0658806c59bc2815b9866"
LABEL eval.benchmark.url="https://huggingface.co/datasets/openai/gsm8k"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2110.14168"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="740312add88f781978c0658806c59bc2815b9866"

# Single-JSONL task layout (see benchmarks/RULES.md #22). One line per task,
# /eval-materialize-task materializes only the requested row at runtime.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/openai/gsm8k/parquet/main/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        full_answer = str(t['answer'][i])
        numeric = full_answer.split('####')[-1].strip().replace(',', '')
        row = {
            'id': str(i),
            'problem': str(t['question'][i]),
            'answer': numeric,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=gsm8k

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this grade school math problem. Print only the final numeric answer as a single number with no units, commas, or explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
