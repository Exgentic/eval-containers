# MBPP+ (EvalPlus augmented MBPP)
# 378 tasks. Data: HuggingFace evalplus/mbppplus
# Agent writes a Python function. Test runs it against the EvalPlus extended test harness.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mbppplus"
LABEL dock.benchmark.description="MBPP+ - EvalPlus augmented MBPP with 35x more tests"
LABEL dock.benchmark.tasks="378"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="b2d74c91837c3f2a20c1299ae98133cbe7cfa077"
LABEL dock.benchmark.url="https://github.com/evalplus/evalplus"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2305.01210"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks
RUN pip install --no-cache-dir pyarrow numpy
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/evalplus/mbppplus/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['task_id'][i]))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['prompt'][i]))
    # test_list is the original MBPP basic asserts used in the prompt
    tl = t['test_list'][i].as_py() or []
    open(f'/tasks/{i}/test_list.txt', 'w').write('\n'.join(tl))
    # test is the full EvalPlus harness (imports + assertion helper + inputs/results driver)
    open(f'/tasks/{i}/test.txt', 'w').write(str(t['test'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=mbppplus

# Test: agent code + EvalPlus harness
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
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

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  TESTS_HINT=""
  [ -f "/tasks/$TASK_ID/test_list.txt" ] && TESTS_HINT="

Your function must satisfy:
$(cat /tasks/$TASK_ID/test_list.txt)"
  export TASK="Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)$TESTS_HINT"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
