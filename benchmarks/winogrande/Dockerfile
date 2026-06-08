# WinoGrande (winogrande_xl validation)
# 1267 tasks. Data: HuggingFace allenai/winogrande (winogrande_xl/validation)
# Agent prints "1" or "2" indicating which option fills the blank.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="winogrande"
LABEL eval.benchmark.description="WinoGrande - large-scale Winograd-style commonsense pronoun resolution (xl validation)"
LABEL eval.benchmark.tasks="1267"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="01e74176c63542e6b0bcb004dcdea22d94fb67b5"
LABEL eval.benchmark.url="https://huggingface.co/datasets/allenai/winogrande"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1907.10641"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="01e74176c63542e6b0bcb004dcdea22d94fb67b5"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/winogrande/resolve/01e74176c63542e6b0bcb004dcdea22d94fb67b5/winogrande_xl/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        sentence = t['sentence'][i].as_py()
        o1 = t['option1'][i].as_py()
        o2 = t['option2'][i].as_py()
        answer = str(t['answer'][i].as_py())
        row = {
            'id': str(i),
            'problem': f"{sentence}\n\n1. {o1}\n2. {o2}",
            'answer': answer,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=winogrande

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="The sentence has a blank marked with _. Choose which option correctly fills the blank. Print only the digit 1 or 2, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
