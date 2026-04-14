# CodeContests (DeepMind competitive programming)
# 165 test tasks from Codeforces/AtCoder/etc. Data: HuggingFace deepmind/code_contests
# Agent writes a solution. Test runs it against public + private + generated I/O tests.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="code-contests"
LABEL dock.benchmark.description="CodeContests - DeepMind competitive programming problems"
LABEL dock.benchmark.tasks="165"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="802411c3010cb00d1b05bad57ca77365a3c699d6"
LABEL dock.benchmark.url="https://github.com/google-deepmind/code_contests"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2203.07814"

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/deepmind/code_contests/resolve/refs%2Fconvert%2Fparquet/default/partial-test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    d = f'/tasks/{i}'
    os.makedirs(d, exist_ok=True)
    os.makedirs(f'{d}/tests', exist_ok=True)
    open(f'{d}/id.txt', 'w').write(str(t['name'][i]))
    open(f'{d}/problem.txt', 'w').write(str(t['description'][i]))
    # Merge public + private + generated tests
    k = 0
    for group in ('public_tests', 'private_tests', 'generated_tests'):
        col = t[group][i].as_py() or {}
        ins = col.get('input') or []
        outs = col.get('output') or []
        for inp, out in zip(ins, outs):
            open(f'{d}/tests/{k}.in', 'w').write(str(inp))
            open(f'{d}/tests/{k}.out', 'w').write(str(out))
            k += 1
    open(f'{d}/num_tests.txt', 'w').write(str(k))
    tl = t['time_limit'][i].as_py() if t['time_limit'][i] else None
    secs = (tl or {}).get('seconds', 10) if isinstance(tl, dict) else 10
    open(f'{d}/time_limit.txt', 'w').write(str(secs))
print(f'Extracted {len(t)} tasks')
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=code-contests

# Test: compile or run agent solution and check against stdio test cases
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
PASSED=0
TOTAL=0

if [ -z "$SOLUTION" ]; then
  echo 0 > /logs/verifier/reward.txt
  exit 0
fi

TMPDIR=$(mktemp -d)
TIME_LIMIT=$(cat $TASK_DIR/time_limit.txt 2>/dev/null)
[ -z "$TIME_LIMIT" ] && TIME_LIMIT=10

if echo "$SOLUTION" | head -5 | grep -q "^#include\|^using namespace"; then
  printf '%s' "$SOLUTION" > $TMPDIR/sol.cpp
  g++ -O2 -o $TMPDIR/sol $TMPDIR/sol.cpp 2>/dev/null || { echo 0 > /logs/verifier/reward.txt; rm -rf $TMPDIR; exit 0; }
  RUN_CMD="$TMPDIR/sol"
else
  printf '%s' "$SOLUTION" > $TMPDIR/sol.py
  RUN_CMD="python3 $TMPDIR/sol.py"
fi

for infile in $TASK_DIR/tests/*.in; do
  [ -f "$infile" ] || continue
  test_id=$(basename "$infile" .in)
  expected=$(cat "$TASK_DIR/tests/${test_id}.out")
  TOTAL=$((TOTAL + 1))
  actual=$(timeout "$TIME_LIMIT" $RUN_CMD < "$infile" 2>/dev/null)
  actual_clean=$(echo "$actual" | tr -d '[:space:]')
  expected_clean=$(echo "$expected" | tr -d '[:space:]')
  [ "$actual_clean" = "$expected_clean" ] && PASSED=$((PASSED + 1))
done

if [ $TOTAL -gt 0 ]; then
  python3 -c "print($PASSED / $TOTAL)" > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
rm -rf $TMPDIR
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competitive programming problem. Read input from stdin and write output to stdout. Print ONLY the complete source code (Python or C++), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
