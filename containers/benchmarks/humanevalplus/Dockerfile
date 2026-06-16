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
RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
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

# Test: pipe prompt + completion + EvalPlus test (ending in `check(<entry>)`) into
# the shared grader /eval-grade (benchmarks/RULES.md 22). Reward is the harness's
# verdict, never the candidate's exit code (rule 5): grade.sh seeds reward.txt
# with a fail and /eval-grade overwrites it with 1.0 only if the program runs to
# completion with no exception. See /eval-grade for the guarantee and residuals.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)
TASK_DIR="/tasks/$EVAL_TASK_ID"
{ cat "$TASK_DIR/prompt.txt"; echo
  cat /output/agent/stdout.log 2>/dev/null; echo
  cat "$TASK_DIR/test.txt"; echo
  echo "check($(cat "$TASK_DIR/entry_point.txt"))"
} | timeout 60 /eval-grade exec /logs/verifier/reward.txt >/dev/null 2>&1 || true
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
