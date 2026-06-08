# HumanEval (OpenAI code generation benchmark)
# 164 tasks. Data: HuggingFace openai/openai_humaneval
# Agent completes a Python function. Test runs it against unit tests.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="humaneval"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="HumanEval - Python code generation"
LABEL eval.benchmark.tasks="164"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="7dce6050a7d6d172f3cc5c32aa97f52fa1a2e544"
LABEL eval.benchmark.url="https://github.com/openai/human-eval"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/humaneval"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2107.03374"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="7dce6050a7d6d172f3cc5c32aa97f52fa1a2e544"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/openai/openai_humaneval/resolve/refs%2Fconvert%2Fparquet/openai_humaneval/test/0000.parquet',
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
ENV EVAL_BENCHMARK=humaneval

# Test: combine agent code with test harness and run
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
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
