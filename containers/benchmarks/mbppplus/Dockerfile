# MBPP+ (EvalPlus augmented MBPP)
# 378 tasks. Data: HuggingFace evalplus/mbppplus
# Agent writes a Python function. Test runs it against the EvalPlus extended test harness.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mbppplus"
LABEL eval.benchmark.description="MBPP+ - EvalPlus augmented MBPP with 35x more tests"
LABEL eval.benchmark.tasks="378"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="b2d74c91837c3f2a20c1299ae98133cbe7cfa077"
LABEL eval.benchmark.url="https://github.com/evalplus/evalplus"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2305.01210"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
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
ENV EVAL_BENCHMARK=mbppplus

# Test: pipe completion + EvalPlus test into the shared grader /eval-grade
# (benchmarks/RULES.md 22). Reward is the harness's verdict, never the candidate's
# exit code (rule 5): grade.sh seeds reward.txt with a fail and /eval-grade
# overwrites it with 1.0 only if the tests run with no exception. See /eval-grade
# for the guarantee and residuals.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)
TASK_DIR="/tasks/$EVAL_TASK_ID"
{ cat /output/agent/stdout.log 2>/dev/null; echo
  cat "$TASK_DIR/test.txt"
} | timeout 60 /eval-grade exec /logs/verifier/reward.txt >/dev/null 2>&1 || true
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TESTS_HINT=""
  if [ -s "/tasks/$EVAL_TASK_ID/test_list.txt" ]; then
    TESTS_HINT="

Your function must satisfy:
$(cat /tasks/$EVAL_TASK_ID/test_list.txt)"
  fi
  export TASK="Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)$TESTS_HINT"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
