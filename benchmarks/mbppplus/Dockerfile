# MBPP+ (EvalPlus augmented MBPP)
# 378 tasks. Data: HuggingFace evalplus/mbppplus
# Agent writes a Python function. Test runs it against the EvalPlus extended test harness.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mbppplus"
LABEL dock.benchmark.description="MBPP+ - EvalPlus augmented MBPP with 35x more tests"
LABEL dock.benchmark.tasks="378"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="b2d74c91837c3f2a20c1299ae98133cbe7cfa077"
LABEL dock.benchmark.url="https://github.com/evalplus/evalplus"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2305.01210"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="b2d74c91837c3f2a20c1299ae98133cbe7cfa077"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN pip install --no-cache-dir pyarrow==18.1.0 numpy==2.0.2
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/evalplus/mbppplus/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        tl = t['test_list'][i].as_py() or []
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['prompt'][i]),
            # test_list is the original MBPP basic asserts used in the prompt
            'test_list': '\n'.join(tl),
            # test is the full EvalPlus harness (imports + assertion helper + inputs/results driver)
            'test': str(t['test'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=mbppplus

# Test: agent code + EvalPlus harness
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
TEST_CODE=$(cat $TASK_DIR/test.txt)

TMPDIR=$(mktemp -d)
cat > $TMPDIR/test_solution.py <<PYEOF
$SOLUTION

$TEST_CODE
PYEOF

# Run with timeout; pass = 1.0, fail = 0.0
if timeout 60 python3 $TMPDIR/test_solution.py >/dev/null 2>&1; then
  echo 1.0 > /logs/verifier/reward.txt
else
  echo 0.0 > /logs/verifier/reward.txt
fi
rm -rf $TMPDIR
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  TESTS_HINT=""
  if [ -s "/tasks/$DOCK_TASK_ID/test_list.txt" ]; then
    TESTS_HINT="

Your function must satisfy:
$(cat /tasks/$DOCK_TASK_ID/test_list.txt)"
  fi
  export TASK="Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)$TESTS_HINT"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
