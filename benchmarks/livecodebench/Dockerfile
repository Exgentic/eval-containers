# LiveCodeBench (livecodebench/code_generation_lite)
# Competitive programming from recent contests (LeetCode, Codeforces, AtCoder).
# Data: HuggingFace livecodebench/code_generation_lite release_v5
# Agent writes solution code. Test runs it against hidden test cases.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="livecodebench"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="LiveCodeBench - competitive programming from recent contests"
LABEL eval.benchmark.tasks="880"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0fe84c3912ea0c4d4a78037083943e8f0c4dd505"
LABEL eval.benchmark.url="https://github.com/LiveCodeBench/LiveCodeBench"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/livecodebench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2403.07974"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="0fe84c3912ea0c4d4a78037083943e8f0c4dd505"

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Single-JSONL task layout (see benchmarks/RULES.md #22). Private test cases
# are stored as a JSON array under the tests_json field; entrypoint decodes
# them into /tasks/$EVAL_TASK_ID/tests/{k}.in and .out after materialization.
RUN python3 <<'PYEOF'
import urllib.request, json, os, base64, zlib, pickle

revision = "0fe84c3912ea0c4d4a78037083943e8f0c4dd505"
base = f"https://huggingface.co/datasets/livecodebench/code_generation_lite/resolve/{revision}"
files = ["test.jsonl", "test2.jsonl", "test3.jsonl", "test4.jsonl", "test5.jsonl"]
rows = []
for f in files:
    urllib.request.urlretrieve(f"{base}/{f}", f"/tmp/{f}")
    with open(f"/tmp/{f}") as fh:
        rows.extend(json.loads(line) for line in fh if line.strip())

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for row_in in rows:
        meta = json.loads(row_in["metadata"]) if row_in.get("metadata") and row_in["metadata"] != "{}" else {}
        ptc_raw = row_in.get("private_test_cases", "")
        tc = []
        if ptc_raw:
            decoded = pickle.loads(zlib.decompress(base64.b64decode(ptc_raw)))
            tc = json.loads(decoded) if isinstance(decoded, str) else decoded
        tests = []
        for t in tc:
            inp = t.get("input", "")
            out_v = t.get("output", "")
            tests.append({
                'input': inp if isinstance(inp, str) else str(inp),
                'output': out_v if isinstance(out_v, str) else str(out_v),
            })
        row = {
            'id': row_in["question_id"],
            'problem': row_in["question_content"],
            'num_tests': str(len(tc)),
            'tests_json': json.dumps(tests),
        }
        if row_in.get("starter_code"):
            row['starter_code'] = row_in["starter_code"]
        if meta.get("func_name"):
            row['fn_name'] = meta["func_name"]
        out.write(json.dumps(row) + '\n')
        count += 1
print(f"Wrote {count} tasks")
PYEOF
RUN rm -f /tmp/*.jsonl
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=livecodebench

# Test: compile agent's solution and run against test cases
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
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
FN_NAME=""
[ -f "$TASK_DIR/fn_name.txt" ] && FN_NAME=$(cat "$TASK_DIR/fn_name.txt")

# Detect language and mode
if echo "$SOLUTION" | head -5 | grep -q "^#include\|^using namespace"; then
  echo "$SOLUTION" > $TMPDIR/sol.cpp
  g++ -O2 -o $TMPDIR/sol $TMPDIR/sol.cpp 2>/dev/null || { echo 0 > /logs/verifier/reward.txt; exit 0; }
  RUN_CMD="$TMPDIR/sol"
  MODE="stdio"
elif [ -n "$FN_NAME" ]; then
  echo "$SOLUTION" > $TMPDIR/sol.py
  MODE="functional"
  RUN_CMD="python3 $TMPDIR/sol.py"
else
  echo "$SOLUTION" > $TMPDIR/sol.py
  MODE="stdio"
  RUN_CMD="python3 $TMPDIR/sol.py"
fi

for infile in $TASK_DIR/tests/*.in; do
  [ -f "$infile" ] || continue
  test_id=$(basename "$infile" .in)
  expected=$(cat "$TASK_DIR/tests/${test_id}.out")
  TOTAL=$((TOTAL + 1))

  if [ "$MODE" = "functional" ]; then
    actual=$(timeout 30 python3 -c "
import json, sys
exec(open('$TMPDIR/sol.py').read())
_inp = json.loads(open('$infile').read())
_args = _inp if isinstance(_inp, list) else [_inp]
if 'Solution' in dir():
    _r = getattr(Solution(), '$FN_NAME')(*_args)
else:
    _r = $FN_NAME(*_args)
print(json.dumps(_r))
" 2>/dev/null)
  else
    actual=$(timeout 30 $RUN_CMD < "$infile" 2>/dev/null)
  fi

  actual_clean=$(echo "$actual" | tr -d '[:space:]')
  expected_clean=$(echo "$expected" | tr -d '[:space:]')
  [ "$actual_clean" = "$expected_clean" ] && PASSED=$((PASSED + 1))
done

if [ $TOTAL -gt 0 ]; then
  python3 -c "print($PASSED / $TOTAL)" > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/tests_json.txt ]; then
  python3 - <<'PY'
import json, os
tid = os.environ['EVAL_TASK_ID']
d = f'/tasks/{tid}'
tests = json.load(open(f'{d}/tests_json.txt'))
os.makedirs(f'{d}/tests', exist_ok=True)
for k, t in enumerate(tests):
    open(f'{d}/tests/{k}.in', 'w').write(t['input'])
    open(f'{d}/tests/{k}.out', 'w').write(t['output'])
PY
  rm -f /tasks/$EVAL_TASK_ID/tests_json.txt
  chmod -R 600 /tasks/$EVAL_TASK_ID/tests
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  STARTER=""
  [ -f "/tasks/$EVAL_TASK_ID/starter_code.txt" ] && STARTER="

Starter code:
$(cat /tasks/$EVAL_TASK_ID/starter_code.txt)"
  export TASK="Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.$STARTER

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
