# USACO (USA Computing Olympiad)
# 307 tasks. Data: HuggingFace dapumptu/usaco_benchmark
# Agent writes solution code. Test runs it against I/O test cases.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22). tests = a JSON string of
# {inputs:[...], outputs:[...]}; the entrypoint explodes it into per-case files.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT problem_id::VARCHAR AS id, description AS problem, CAST(num_tests AS VARCHAR) AS num_tests, \
            to_json({'inputs': input, 'outputs': output})::VARCHAR AS tests \
     FROM read_parquet('hf://datasets/dapumptu/usaco_benchmark@~parquet/default/train/*.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

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

RUN apt-get update && apt-get install -y --no-install-recommends gcc g++ && rm -rf /var/lib/apt/lists/*

COPY --from=data /tasks /tasks
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
