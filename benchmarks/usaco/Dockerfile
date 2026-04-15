# USACO (USA Computing Olympiad)
# 307 tasks. Data: HuggingFace dapumptu/usaco_benchmark
# Agent writes solution code. Test runs it against I/O test cases.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="usaco"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="USACO - competitive programming"
LABEL dock.benchmark.tasks="307"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://huggingface.co/datasets/dapumptu/usaco_benchmark"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/usaco"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/usaco"
ENV DOCK_BENCHMARK_VERSION_DEFAULT=""

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/,
# then explodes the per-case inputs/outputs dict into /tasks/$DOCK_TASK_ID/tests/.
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
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
ENV DOCK_BENCHMARK=usaco

# Test: compile agent's solution and run against test cases
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
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
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
# Explode the packed tests JSON back into per-case .in/.out files under tests/.
if [ -n "$DOCK_TASK_ID" ] && [ -f /tasks/$DOCK_TASK_ID/tests.txt ]; then
  python3 - <<'PY'
import json, os
tid = os.environ['DOCK_TASK_ID']
with open(f'/tasks/{tid}/tests.txt') as f:
    data = json.loads(f.read())
td = f'/tasks/{tid}/tests'
os.makedirs(td, exist_ok=True)
for k, v in data.get('inputs', {}).items():
    open(f'{td}/{k}.in', 'w').write(str(v))
for k, v in data.get('outputs', {}).items():
    open(f'{td}/{k}.out', 'w').write(str(v))
PY
  rm -f /tasks/$DOCK_TASK_ID/tests.txt
  chmod -R 600 /tasks/$DOCK_TASK_ID/tests
fi
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
