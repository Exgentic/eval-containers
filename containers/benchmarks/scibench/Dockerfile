# SciBench (college-level scientific problems)
# 692 tasks. Data: HuggingFace xw27/scibench
# Agent prints answer to stdout. Test compares to expected numeric answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT trim(CAST(problemid AS VARCHAR)) AS id, problem_text AS problem, \
            trim(CAST(answer_number AS VARCHAR)) AS answer, unit, source \
     FROM read_parquet('hf://datasets/xw27/scibench@~parquet/default/train/0000.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="scibench"
LABEL eval.benchmark.description="SciBench - college-level scientific problems in math, physics, and chemistry"
LABEL eval.benchmark.tasks="692"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="93931252bc1b71d495e67390235940643d926958"
LABEL eval.benchmark.url="https://github.com/mandyyyyii/scibench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2307.10635"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=scibench
ENV EVAL_TASK_PROMPT="Solve this college-level science problem. Print only the final numeric answer as a single number (no units, no explanation). Expected unit: {unit}."
