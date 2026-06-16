# USACO (USA Computing Olympiad)
# 307 tasks. Data: HuggingFace dapumptu/usaco_benchmark
# Agent writes solution code. Test runs it against I/O test cases.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="usaco"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="USACO - competitive programming"
LABEL eval.benchmark.tasks="307"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://huggingface.co/datasets/dapumptu/usaco_benchmark"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/usaco"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/usaco"

RUN apt-get update && apt-get install -y --no-install-recommends curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/,
# then explodes the per-case inputs/outputs dict into /tasks/$EVAL_TASK_ID/tests/.
RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, pyarrow, os, json
tables = []
for part in range(3):
    url = f'https://huggingface.co/datasets/dapumptu/usaco_benchmark/resolve/refs%2Fconvert%2Fparquet/default/train/000{part}.parquet'
    urllib.request.urlretrieve(url, f'/tmp/data_{part}.parquet')
    tables.append(pq.read_table(f'/tmp/data_{part}.parquet'))
t = pyarrow.concat_tables(tables)
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        inputs = t['input'][i].as_py()
        outputs = t['output'][i].as_py()
        row = {
            'id': str(t['problem_id'][i]),
            'problem': str(t['description'][i]),
            'num_tests': str(t['num_tests'][i]),
            'tests': json.dumps({'inputs': inputs, 'outputs': outputs}),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=usaco

# Test: compile agent's solution and run against test cases
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
PASSED=0
TOTAL=0

# Try to detect language and compile/run
TMPDIR=$(mktemp -d)
if echo "$SOLUTION" | head -5 | grep -q "^#include\|^using namespace"; then
  echo "$SOLUTION" > $TMPDIR/sol.cpp
  g++ -O2 -o $TMPDIR/sol $TMPDIR/sol.cpp 2>/dev/null || { echo 0 > /logs/verifier/reward.txt; exit 0; }
  RUN_CMD="$TMPDIR/sol"
else
  echo "$SOLUTION" > $TMPDIR/sol.py
  RUN_CMD="python3 $TMPDIR/sol.py"
fi

for infile in $TASK_DIR/tests/*.in; do
  test_id=$(basename $infile .in)
  expected=$(cat $TASK_DIR/tests/${test_id}.out)
  actual=$(timeout 10 $RUN_CMD < $infile 2>/dev/null | tr -d '[:space:]')
  expected_clean=$(echo "$expected" | tr -d '[:space:]')
  TOTAL=$((TOTAL + 1))
  [ "$actual" = "$expected_clean" ] && PASSED=$((PASSED + 1))
done

if [ $TOTAL -gt 0 ]; then
  python3 -c "print($PASSED / $TOTAL)" > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
# Explode the packed tests JSON back into per-case .in/.out files under tests/.
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/tests.txt ]; then
  python3 - <<'PY'
import json, os
tid = os.environ['EVAL_TASK_ID']
with open(f'/tasks/{tid}/tests.txt') as f:
    data = json.loads(f.read())
td = f'/tasks/{tid}/tests'
os.makedirs(td, exist_ok=True)
for k, v in data.get('inputs', {}).items():
    open(f'{td}/{k}.in', 'w').write(str(v))
for k, v in data.get('outputs', {}).items():
    open(f'{td}/{k}.out', 'w').write(str(v))
PY
  rm -f /tasks/$EVAL_TASK_ID/tests.txt
  chmod -R 600 /tasks/$EVAL_TASK_ID/tests
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
