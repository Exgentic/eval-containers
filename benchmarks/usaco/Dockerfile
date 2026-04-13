# USACO (USA Computing Olympiad)
# 307 tasks. Data: HuggingFace dapumptu/usaco_benchmark
# Agent writes solution code. Test runs it against I/O test cases.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="usaco"
LABEL dock.benchmark.description="USACO - competitive programming"
LABEL dock.benchmark.tasks="307"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://huggingface.co/datasets/dapumptu/usaco_benchmark"

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Fetch tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, pyarrow, os, json
tables = []
for part in range(3):
    url = f'https://huggingface.co/datasets/dapumptu/usaco_benchmark/resolve/refs%2Fconvert%2Fparquet/default/train/000{part}.parquet'
    urllib.request.urlretrieve(url, f'/tmp/data_{part}.parquet')
    tables.append(pq.read_table(f'/tmp/data_{part}.parquet'))
t = pyarrow.concat_tables(tables)
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt','w').write(str(t['problem_id'][i]))
    open(f'/tasks/{i}/problem.txt','w').write(str(t['description'][i]))
    # Save test cases as numbered input/output files
    inputs = t['input'][i].as_py()
    outputs = t['output'][i].as_py()
    os.makedirs(f'/tasks/{i}/tests', exist_ok=True)
    for k in inputs:
        open(f'/tasks/{i}/tests/{k}.in','w').write(str(inputs[k]))
        open(f'/tasks/{i}/tests/{k}.out','w').write(str(outputs[k]))
    open(f'/tasks/{i}/num_tests.txt','w').write(str(t['num_tests'][i]))
PYEOF
RUN rm -f /tmp/data_*.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=usaco

# Test: compile agent's solution and run against test cases
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
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

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
