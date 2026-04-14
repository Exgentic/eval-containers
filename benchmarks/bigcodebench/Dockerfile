# BigCodeBench (BigCode)
# 1140 tasks. Data: HuggingFace bigcode/bigcodebench (v0.1.4)
# Agent writes a complete Python function. Test runs upstream unittest harness.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="bigcodebench"
LABEL dock.benchmark.description="BigCodeBench - practical Python function generation with rich tool calls"
LABEL dock.benchmark.tasks="1140"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="b74c0d0bf70d2c0bc459be537895cca163007f1a"
LABEL dock.benchmark.url="https://github.com/bigcode-project/bigcodebench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2406.15877"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bigcodebench"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/bigcode/bigcodebench/resolve/b74c0d0bf70d2c0bc459be537895cca163007f1a/data/v0.1.4-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['task_id'][i]))
    open(f'/tasks/{i}/instruct_prompt.txt', 'w').write(str(t['instruct_prompt'][i]))
    open(f'/tasks/{i}/code_prompt.txt', 'w').write(str(t['code_prompt'][i]))
    open(f'/tasks/{i}/test.txt', 'w').write(str(t['test'][i]))
    open(f'/tasks/{i}/entry_point.txt', 'w').write(str(t['entry_point'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=bigcodebench

# Test: combine agent solution with upstream test and run unittest
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
TEST_CODE=$(cat $TASK_DIR/test.txt)

TMPDIR=$(mktemp -d)
{
  echo "$SOLUTION"
  echo
  echo "$TEST_CODE"
  echo
  echo "import unittest"
  echo "if __name__ == '__main__':"
  echo "    loader = unittest.TestLoader()"
  echo "    suite = loader.loadTestsFromTestCase(TestCases)"
  echo "    runner = unittest.TextTestRunner(verbosity=0)"
  echo "    result = runner.run(suite)"
  echo "    import sys; sys.exit(0 if result.wasSuccessful() else 1)"
} > $TMPDIR/test_solution.py

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
  export TASK="Write a complete, self-contained Python solution for the following task. Print ONLY the Python code (including all needed imports and the full function definition), nothing else.

$(cat /tasks/$TASK_ID/instruct_prompt.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
