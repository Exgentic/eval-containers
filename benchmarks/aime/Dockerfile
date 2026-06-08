# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime
# Agent prints answer to stdout. Test compares stdout to expected answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="aime"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="American Invitational Mathematics Examination"
LABEL eval.benchmark.tasks="90"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="13f9e12f613e720c2a2b2f345dd04b998a29494d"
LABEL eval.benchmark.url="https://huggingface.co/datasets/AI-MO/aimo-validation-aime"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/aime"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/aime2024"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="13f9e12f613e720c2a2b2f345dd04b998a29494d"

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
ENV EVAL_BENCHMARK=aime

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this problem. Print only the answer as a single integer.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
