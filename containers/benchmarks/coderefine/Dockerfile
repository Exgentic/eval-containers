# CodeRefine (CodeXGLUE code-refinement, medium)
# 6545 Java buggy->fixed pairs. Data: HuggingFace google/code_x_glue_cc_code_refinement
# Agent is given a buggy Java function and must print the refined (bug-free) version.
# Test compares stdout to the reference fix (exact match; upstream follows the same
# convention — all identifiers are normalized).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT CAST(id AS VARCHAR) AS id, buggy AS problem, fixed AS answer \
     FROM read_parquet('hf://datasets/google/code_x_glue_cc_code_refinement@~parquet/medium/test/0000.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="coderefine"
LABEL eval.benchmark.description="CodeRefine (CodeXGLUE) - Java bug-fix code refinement"
LABEL eval.benchmark.tasks="6545"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="07ab797a018d0d5c448b56eb26b5e11aa5ad7659"
LABEL eval.benchmark.url="https://github.com/microsoft/CodeXGLUE/tree/main/Code-Code/code-refinement"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2102.04664"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=coderefine

# Bespoke entrypoint: the prompt embeds the buggy code under a "Buggy code:" label
# with a single newline, which the shared "{prompt}\n\n{problem}" layout can't
# reproduce byte-for-byte, so keep the original entrypoint verbatim.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are given a buggy Java function. Print the refined (bug-free)
version as a single line of Java, identifiers normalized. Print only the fixed code,
nothing else.

Buggy code:
$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
