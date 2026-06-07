# TheoremQA (theorem-driven question answering)
# 800 tasks. Data: HuggingFace TIGER-Lab/TheoremQA
# Agent prints answer to stdout. Test compares to expected answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="theoremqa"
LABEL eval.benchmark.description="TheoremQA - STEM problems requiring the application of named theorems"
LABEL eval.benchmark.tasks="800"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="a340b1782960a712843aae3ed25f1e013cc008a5"
LABEL eval.benchmark.url="https://huggingface.co/datasets/TIGER-Lab/TheoremQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2305.12524"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="a340b1782960a712843aae3ed25f1e013cc008a5"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/TIGER-Lab/TheoremQA/parquet/default/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(i),
            'problem': str(t['Question'][i]),
            'answer': str(t['Answer'][i]),
            'answer_type': str(t['Answer_type'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=theoremqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this theorem-based problem. Print only the final answer (a number, expression, or value). Do not include any explanation or units.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
