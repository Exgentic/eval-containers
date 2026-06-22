# BigCodeBench (BigCode)
# 1140 tasks. Data: HuggingFace bigcode/bigcodebench (v0.1.4)
# Agent writes a complete Python function. Test runs upstream unittest harness.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT task_id AS id, instruct_prompt, code_prompt, test, entry_point, canonical_solution \
     FROM 'hf://datasets/bigcode/bigcodebench@b74c0d0bf70d2c0bc459be537895cca163007f1a/data/v0.1.4-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="bigcodebench"
LABEL eval.benchmark.description="BigCodeBench - practical Python function generation with rich tool calls"
LABEL eval.benchmark.tasks="1140"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="b74c0d0bf70d2c0bc459be537895cca163007f1a"
LABEL eval.benchmark.url="https://github.com/bigcode-project/bigcodebench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.15877"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bigcodebench"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=bigcodebench

# Test: pipe solution + upstream test into the shared grader /eval-grade
# (benchmarks/RULES.md 22). Reward is the harness's verdict, never the candidate's
# exit code (rule 5): grade.sh seeds reward.txt with a fail and /eval-grade
# overwrites it with 1.0 only on a genuine unittest pass (read from TestResult,
# not the patchable runner). See /eval-grade for the guarantee and residuals.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo 0.0 > /logs/verifier/reward.txt          # default-fail (rule 18)
TASK_DIR="/tasks/$EVAL_TASK_ID"
{ cat /output/agent/stdout.log 2>/dev/null; echo
  cat "$TASK_DIR/test.txt"
} | timeout 90 /eval-grade unittest /logs/verifier/reward.txt >/dev/null 2>&1 || true
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Write a complete, self-contained Python solution for the following task. Print ONLY the Python code (including all needed imports and the full function definition), nothing else.

$(cat /tasks/$EVAL_TASK_ID/instruct_prompt.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
