# CodeRefine (CodeXGLUE code-refinement, medium)
# 6545 Java buggy->fixed pairs. Data: HuggingFace google/code_x_glue_cc_code_refinement
# Agent is given a buggy Java function and must print the refined (bug-free) version.
# Test compares stdout to the reference fix (exact match; upstream follows the same
# convention — all identifiers are normalized).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="coderefine"
LABEL dock.benchmark.description="CodeRefine (CodeXGLUE) - Java bug-fix code refinement"
LABEL dock.benchmark.tasks="6545"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="07ab797a018d0d5c448b56eb26b5e11aa5ad7659"
LABEL dock.benchmark.url="https://github.com/microsoft/CodeXGLUE/tree/main/Code-Code/code-refinement"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2102.04664"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/google/code_x_glue_cc_code_refinement/parquet/medium/test/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt','w').write(str(t['id'][i]))
    open(f'/tasks/{i}/problem.txt','w').write(str(t['buggy'][i]))
    open(f'/tasks/{i}/answer.txt','w').write(str(t['fixed'][i]))
print(f'Extracted {len(t)} tasks')
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=coderefine

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are given a buggy Java function. Print the refined (bug-free)
version as a single line of Java, identifiers normalized. Print only the fixed code,
nothing else.

Buggy code:
$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
