# SimpleQA (OpenAI)
# 4326 tasks. Data: HuggingFace basicv8vc/SimpleQA
# Agent prints answer to stdout. Test compares stdout to expected answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="simpleqa"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="SimpleQA - factual question answering"
LABEL eval.benchmark.tasks="4326"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="e319282ab125c3dbd0c7fd00be2e4dd54e7e8f94"
LABEL eval.benchmark.url="https://github.com/openai/simple-evals"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/simpleqa"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/simpleqa"
LABEL eval.benchmark.paper="https://openai.com/index/introducing-simpleqa/"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve('https://huggingface.co/datasets/basicv8vc/SimpleQA/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(i),
            'problem': str(t['problem'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=simpleqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
