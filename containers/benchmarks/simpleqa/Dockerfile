# SimpleQA (OpenAI)
# 4326 tasks. Data: HuggingFace basicv8vc/SimpleQA
# Agent prints answer to stdout. Test compares stdout to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, problem, answer \
     FROM 'hf://datasets/basicv8vc/SimpleQA@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="simpleqa"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="SimpleQA - factual question answering"
LABEL eval.benchmark.tasks="4326"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="e319282ab125c3dbd0c7fd00be2e4dd54e7e8f94"
LABEL eval.benchmark.url="https://github.com/openai/simple-evals"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/simpleqa"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/simpleqa"
LABEL eval.benchmark.paper="https://openai.com/index/introducing-simpleqa/"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=simpleqa
ENV EVAL_TASK_PROMPT="Answer this question. Print only the answer, nothing else."
