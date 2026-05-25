# MATH-500 (Competition-level mathematics)
# 500 tasks. Data: HuggingFace HuggingFaceH4/MATH-500
# Agent prints answer to stdout. Test compares to expected answer.

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="math-500"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MATH-500 - competition-level math problems"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="6e4ed1a2a79af7d8630a6b768ec859cb5af4d3be"
LABEL eval.benchmark.url="https://github.com/hendrycks/math"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/math"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2103.03874"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="6e4ed1a2a79af7d8630a6b768ec859cb5af4d3be"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/HuggingFaceH4/MATH-500/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['unique_id'][i]),
            'problem': str(t['problem'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=math-500

COPY --from=test-exact-match /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this math problem. Print only the final answer in its simplest form (e.g. a number, fraction, or expression). Do not include any explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
