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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="b74c0d0bf70d2c0bc459be537895cca163007f1a"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Single-JSONL task layout. Agent never sees /tasks/all.jsonl (root-only).
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/bigcode/bigcodebench/resolve/b74c0d0bf70d2c0bc459be537895cca163007f1a/data/v0.1.4-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        # "problem" is a synonym used elsewhere for the agent-facing prompt;
        # here we preserve BigCodeBench's original field name "instruct_prompt".
        row = {
            'id': str(t['task_id'][i]),
            'instruct_prompt': str(t['instruct_prompt'][i]),
            'code_prompt': str(t['code_prompt'][i]),
            'test': str(t['test'][i]),
            'entry_point': str(t['entry_point'][i]),
        }
        out.write(json.dumps(row) + '\n')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=bigcodebench

# Test: combine agent solution with upstream test and run unittest
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
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

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Write a complete, self-contained Python solution for the following task. Print ONLY the Python code (including all needed imports and the full function definition), nothing else.

$(cat /tasks/$DOCK_TASK_ID/instruct_prompt.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
