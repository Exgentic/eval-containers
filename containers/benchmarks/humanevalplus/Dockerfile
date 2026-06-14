# HumanEval+ (EvalPlus augmented HumanEval)
# 164 tasks. Data: HuggingFace evalplus/humanevalplus
# Agent completes a Python function. Test runs it against the EvalPlus extended test harness.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="humanevalplus"
LABEL eval.benchmark.description="HumanEval+ - EvalPlus augmented HumanEval with 80x more tests"
LABEL eval.benchmark.tasks="164"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="d32357cf319e50e9c8d8dab5ea876c72b0fd321b"
LABEL eval.benchmark.url="https://github.com/evalplus/evalplus"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2305.01210"

RUN pip install --no-cache-dir pyarrow==18.1.0 numpy==2.0.2

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/evalplus/humanevalplus/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['prompt'][i]),
            'prompt': str(t['prompt'][i]),
            'test': str(t['test'][i]),
            'entry_point': str(t['entry_point'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=humanevalplus

# Test: combine agent code with EvalPlus test harness and run
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)
PROMPT=$(cat $TASK_DIR/prompt.txt)
TEST_CODE=$(cat $TASK_DIR/test.txt)
ENTRY_POINT=$(cat $TASK_DIR/entry_point.txt)

# Build the full program: prompt + agent completion + test harness + check call
TMPDIR=$(mktemp -d)
cat > $TMPDIR/test_solution.py <<PYEOF
$PROMPT
$SOLUTION

$TEST_CODE

check($ENTRY_POINT)
PYEOF

# Run with timeout; pass = 1.0, fail = 0.0
if timeout 60 python3 $TMPDIR/test_solution.py >/dev/null 2>&1; then
  echo 1.0 > /logs/verifier/reward.txt
else
  echo 0.0 > /logs/verifier/reward.txt
fi
rm -rf $TMPDIR
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Complete the following Python function. Print ONLY the function body (the code that goes after the function signature), nothing else. Do not repeat the function signature or docstring.

$(cat /tasks/$EVAL_TASK_ID/prompt.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
