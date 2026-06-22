# MBPP (Mostly Basic Python Problems)
# 500 tasks. Data: HuggingFace google-research-datasets/mbpp
# Agent writes a Python function. Test runs it against assert-based test cases.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT task_id::VARCHAR AS id, text AS problem, \
            array_to_string(test_list, chr(10)) AS tests, \
            COALESCE(test_setup_code, '') AS setup, code \
     FROM 'hf://datasets/google-research-datasets/mbpp@~parquet/full/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mbpp"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MBPP - mostly basic Python problems"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="4bb6404fdc6cacfda99d4ac4205087b89d32030c"
LABEL eval.benchmark.url="https://github.com/google-research/google-research/tree/master/mbpp"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mbpp"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2108.07732"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=mbpp

# Test: pipe setup + completion + assert-based tests into the shared grader
# /eval-grade (benchmarks/RULES.md 22). Reward is the harness's verdict, never the
# candidate's exit code (rule 5): grade.sh seeds reward.txt with a fail and
# /eval-grade overwrites it with 1.0 only if every assert runs with no exception.
# See /eval-grade for the guarantee and residuals.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)
TASK_DIR="/tasks/$EVAL_TASK_ID"
{ cat "$TASK_DIR/setup.txt" 2>/dev/null; echo
  cat /output/agent/stdout.log 2>/dev/null; echo
  cat "$TASK_DIR/tests.txt" 2>/dev/null
} | timeout 30 /eval-grade exec /logs/verifier/reward.txt >/dev/null 2>&1 || true
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
