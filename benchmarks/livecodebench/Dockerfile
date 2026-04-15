# LiveCodeBench (livecodebench/code_generation_lite)
# Competitive programming from recent contests (LeetCode, Codeforces, AtCoder).
# Data: HuggingFace livecodebench/code_generation_lite release_v5
# Agent writes solution code. Test runs it against hidden test cases.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="livecodebench"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="LiveCodeBench - competitive programming from recent contests"
LABEL dock.benchmark.tasks="880"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="0fe84c3912ea0c4d4a78037083943e8f0c4dd505"
LABEL dock.benchmark.url="https://github.com/LiveCodeBench/LiveCodeBench"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/livecodebench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2403.07974"

RUN apt-get update && apt-get install -y curl gcc g++ && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks from JSONL files (release_v5 = test through test5)
RUN python3 <<'PYEOF'
import urllib.request, json, os, base64, zlib, pickle

revision = "0fe84c3912ea0c4d4a78037083943e8f0c4dd505"
base = f"https://huggingface.co/datasets/livecodebench/code_generation_lite/resolve/{revision}"
# release_v5 includes test.jsonl through test5.jsonl
files = ["test.jsonl", "test2.jsonl", "test3.jsonl", "test4.jsonl", "test5.jsonl"]
rows = []
for f in files:
    urllib.request.urlretrieve(f"{base}/{f}", f"/tmp/{f}")
    with open(f"/tmp/{f}") as fh:
        rows.extend(json.loads(line) for line in fh if line.strip())
for i, row in enumerate(rows):
    d = f"/tasks/{i}"
    os.makedirs(d, exist_ok=True)
    os.makedirs(f"{d}/tests", exist_ok=True)
    open(f"{d}/id.txt", "w").write(row["question_id"])
    open(f"{d}/problem.txt", "w").write(row["question_content"])
    if row.get("starter_code"):
        open(f"{d}/starter_code.txt", "w").write(row["starter_code"])
    # Parse metadata for fn_name
    meta = json.loads(row["metadata"]) if row.get("metadata") and row["metadata"] != "{}" else {}
    if meta.get("func_name"):
        open(f"{d}/fn_name.txt", "w").write(meta["func_name"])
    # Decode private test cases: base64 -> zlib -> pickle -> JSON string
    ptc_raw = row.get("private_test_cases", "")
    tc = []
    if ptc_raw:
        decoded = pickle.loads(zlib.decompress(base64.b64decode(ptc_raw)))
        tc = json.loads(decoded) if isinstance(decoded, str) else decoded
    for k, t in enumerate(tc):
        inp = t.get("input", "")
        out = t.get("output", "")
        open(f"{d}/tests/{k}.in", "w").write(str(inp) if not isinstance(inp, str) else inp)
        open(f"{d}/tests/{k}.out", "w").write(str(out) if not isinstance(out, str) else out)
    open(f"{d}/num_tests.txt", "w").write(str(len(tc)))
print(f"Extracted {len(rows)} tasks")
PYEOF
RUN rm -f /tmp/*.jsonl
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=livecodebench

# Test: compile agent's solution and run against test cases
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
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

  # Compare: strip whitespace for both
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

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  STARTER=""
  [ -f "/tasks/$DOCK_TASK_ID/starter_code.txt" ] && STARTER="

Starter code:
$(cat /tasks/$DOCK_TASK_ID/starter_code.txt)"
  export TASK="Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.$STARTER

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
