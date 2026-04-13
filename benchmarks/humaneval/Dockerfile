# HumanEval (OpenAI code generation benchmark)
# 164 tasks. Data: HuggingFace openai/openai_humaneval
# Agent completes a Python function. Test runs it against unit tests.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="humaneval"
LABEL dock.benchmark.description="HumanEval - Python code generation"
LABEL dock.benchmark.tasks="164"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="7dce6050a7d6d172f3cc5c32aa97f52fa1a2e544"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/openai/openai_humaneval/resolve/refs%2Fconvert%2Fparquet/openai_humaneval/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['task_id'][i]))
    open(f'/tasks/{i}/prompt.txt', 'w').write(str(t['prompt'][i]))
    open(f'/tasks/{i}/test.txt', 'w').write(str(t['test'][i]))
    open(f'/tasks/{i}/entry_point.txt', 'w').write(str(t['entry_point'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=humaneval

# Test: combine agent code with test harness and run
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
PROMPT=$(cat $TASK_DIR/prompt.txt)
TEST_CODE=$(cat $TASK_DIR/test.txt)
ENTRY_POINT=$(cat $TASK_DIR/entry_point.txt)

# Build the full program: prompt + agent completion + test call
TMPDIR=$(mktemp -d)
cat > $TMPDIR/test_solution.py <<PYEOF
$PROMPT
$SOLUTION

$TEST_CODE

check($ENTRY_POINT)
PYEOF

# Run with timeout; pass = 1.0, fail = 0.0
if timeout 30 python3 $TMPDIR/test_solution.py >/dev/null 2>&1; then
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
  export TASK="Complete the following Python function. Print ONLY the function body (the code that goes after the function signature), nothing else. Do not repeat the function signature or docstring.

$(cat /tasks/$TASK_ID/prompt.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
