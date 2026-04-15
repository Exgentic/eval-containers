# MBPP (Mostly Basic Python Problems)
# 500 tasks. Data: HuggingFace google-research-datasets/mbpp
# Agent writes a Python function. Test runs it against assert-based test cases.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mbpp"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="MBPP - mostly basic Python problems"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="4bb6404fdc6cacfda99d4ac4205087b89d32030c"
LABEL dock.benchmark.url="https://github.com/google-research/google-research/tree/master/mbpp"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mbpp"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2108.07732"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="4bb6404fdc6cacfda99d4ac4205087b89d32030c"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/google-research-datasets/mbpp/resolve/refs%2Fconvert%2Fparquet/full/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        tests = t['test_list'][i].as_py()
        setup = str(t['test_setup_code'][i]) if t['test_setup_code'][i] else ''
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['text'][i]),
            'tests': '\n'.join(tests),
            'setup': setup,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=mbpp

# Test: run agent's code then execute assert-based test cases against it
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
SETUP=$(cat $TASK_DIR/setup.txt 2>/dev/null)
TESTS=$(cat $TASK_DIR/tests.txt 2>/dev/null)

# Build test program: setup + agent code + assert statements
TMPDIR=$(mktemp -d)
cat > $TMPDIR/test_solution.py <<PYEOF
$SETUP
$SOLUTION

$TESTS
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

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
