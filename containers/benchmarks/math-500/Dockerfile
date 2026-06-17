# MATH-500 (Competition-level mathematics)
# 500 tasks. Data: HuggingFace HuggingFaceH4/MATH-500
# Agent prints answer to stdout. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT unique_id AS id, problem, answer \
     FROM 'hf://datasets/HuggingFaceH4/MATH-500@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="math-500"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MATH-500 - competition-level math problems"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="6e4ed1a2a79af7d8630a6b768ec859cb5af4d3be"
LABEL eval.benchmark.url="https://github.com/hendrycks/math"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/math"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2103.03874"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=math-500
ENV EVAL_TASK_PROMPT="Solve this math problem. Print only the final answer in its simplest form (e.g. a number, fraction, or expression). Do not include any explanation."
