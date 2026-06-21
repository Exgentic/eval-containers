# HumanEval (OpenAI code generation benchmark)
# 164 tasks. Data: HuggingFace openai/openai_humaneval
# Agent completes a Python function. Test runs it against unit tests.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT task_id AS id, prompt AS problem, prompt, test, entry_point \
     FROM 'hf://datasets/openai/openai_humaneval@~parquet/openai_humaneval/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

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

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=humaneval

# Test: pipe prompt + completion + upstream test (ending in `check(<entry>)`) into
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
} | timeout 30 /eval-grade exec /logs/verifier/reward.txt >/dev/null 2>&1 || true
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
