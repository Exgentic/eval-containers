# CodeContests (DeepMind competitive programming)
# 165 test tasks from Codeforces/AtCoder/etc. Data: HuggingFace deepmind/code_contests
# Agent writes a solution. Test runs it against public + private + generated I/O tests.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="code-contests"
LABEL eval.benchmark.description="CodeContests - DeepMind competitive programming problems"
LABEL eval.benchmark.tasks="165"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="802411c3010cb00d1b05bad57ca77365a3c699d6"
LABEL eval.benchmark.url="https://github.com/google-deepmind/code_contests"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2203.07814"

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Single-JSONL task layout. Merged I/O tests are stored as a JSON array in the
# "tests_json" field; the entrypoint expands them into /tasks/$EVAL_TASK_ID/tests/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/deepmind/code_contests/resolve/refs%2Fconvert%2Fparquet/default/partial-test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        tests = []
        for group in ('public_tests', 'private_tests', 'generated_tests'):
            col = t[group][i].as_py() or {}
            ins = col.get('input') or []
            outs = col.get('output') or []
            for inp, o in zip(ins, outs):
                tests.append({'input': str(inp), 'output': str(o)})
        tl = t['time_limit'][i].as_py() if t['time_limit'][i] else None
        secs = (tl or {}).get('seconds', 10) if isinstance(tl, dict) else 10
        row = {
            'id': str(t['name'][i]),
            'problem': str(t['description'][i]),
            'num_tests': str(len(tests)),
            'time_limit': str(secs),
            'tests_json': json.dumps(tests),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=code-contests

# Test: compile or run agent solution and check against stdio test cases
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
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

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/tests_json.txt ]; then
  python3 - <<'PY'
import json, os
tid = os.environ['EVAL_TASK_ID']
with open(f'/tasks/{tid}/tests_json.txt') as f:
    tests = json.loads(f.read())
os.makedirs(f'/tasks/{tid}/tests', exist_ok=True)
for k, tc in enumerate(tests):
    with open(f'/tasks/{tid}/tests/{k}.in', 'w') as fi:
        fi.write(tc['input'])
    with open(f'/tasks/{tid}/tests/{k}.out', 'w') as fo:
        fo.write(tc['output'])
PY
  rm -f /tasks/$EVAL_TASK_ID/tests_json.txt
  chmod -R 600 /tasks/$EVAL_TASK_ID/tests
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this competitive programming problem. Read input from stdin and write output to stdout. Print ONLY the complete source code (Python or C++), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
