# HumanEval+ (EvalPlus augmented HumanEval)
# 164 tasks. Data: HuggingFace evalplus/humanevalplus
# Agent completes a Python function. Test runs it against the EvalPlus extended test harness.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT task_id AS id, prompt AS problem, prompt, test, entry_point, canonical_solution \
     FROM 'hf://datasets/evalplus/humanevalplus@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="humanevalplus"
LABEL eval.benchmark.description="HumanEval+ - EvalPlus augmented HumanEval with 80x more tests"
LABEL eval.benchmark.tasks="164"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="d32357cf319e50e9c8d8dab5ea876c72b0fd321b"
LABEL eval.benchmark.url="https://github.com/evalplus/evalplus"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2305.01210"

# numpy for EvalPlus test execution (pyarrow was loader-only, now in the builder).
RUN pip install --no-cache-dir numpy==2.0.2
COPY --from=data /tasks /tasks
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
