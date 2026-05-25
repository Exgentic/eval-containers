# CodeRefine (CodeXGLUE code-refinement, medium)
# 6545 Java buggy->fixed pairs. Data: HuggingFace google/code_x_glue_cc_code_refinement
# Agent is given a buggy Java function and must print the refined (bug-free) version.
# Test compares stdout to the reference fix (exact match; upstream follows the same
# convention — all identifiers are normalized).

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="coderefine"
LABEL eval.benchmark.description="CodeRefine (CodeXGLUE) - Java bug-fix code refinement"
LABEL eval.benchmark.tasks="6545"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="07ab797a018d0d5c448b56eb26b5e11aa5ad7659"
LABEL eval.benchmark.url="https://github.com/microsoft/CodeXGLUE/tree/main/Code-Code/code-refinement"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2102.04664"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="07ab797a018d0d5c448b56eb26b5e11aa5ad7659"

RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/google/code_x_glue_cc_code_refinement/parquet/medium/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['id'][i]),
            'problem': str(t['buggy'][i]),
            'answer': str(t['fixed'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=coderefine

COPY --from=test-exact-match /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are given a buggy Java function. Print the refined (bug-free)
version as a single line of Java, identifiers normalized. Print only the fixed code,
nothing else.

Buggy code:
$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
