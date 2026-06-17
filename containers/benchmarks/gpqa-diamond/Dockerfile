# GPQA Diamond
# 198 tasks. Data: HuggingFace fingertap/GPQA-Diamond
# Agent prints answer to stdout. Test compares stdout to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, question AS problem, answer \
     FROM 'hf://datasets/fingertap/GPQA-Diamond@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="gpqa-diamond"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="GPQA Diamond - graduate-level science"
LABEL eval.benchmark.tasks="198"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="68be7564497676e07a77a042fdb587deb88c51c3"
LABEL eval.benchmark.url="https://github.com/idavidrein/gpqa"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/gpqa-diamond"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/gpqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2311.12022"
COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=gpqa-diamond
ENV EVAL_TASK_PROMPT="Answer this multiple choice question. Print only the letter (A, B, C, or D), nothing else."
