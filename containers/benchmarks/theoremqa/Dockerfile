# TheoremQA (theorem-driven question answering)
# 800 tasks. Data: HuggingFace TIGER-Lab/TheoremQA
# Agent prints answer to stdout. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, Question AS problem, \
     Answer::VARCHAR AS answer, Answer_type::VARCHAR AS answer_type \
     FROM 'hf://datasets/TIGER-Lab/TheoremQA@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="theoremqa"
LABEL eval.benchmark.description="TheoremQA - STEM problems requiring the application of named theorems"
LABEL eval.benchmark.tasks="800"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="a340b1782960a712843aae3ed25f1e013cc008a5"
LABEL eval.benchmark.url="https://huggingface.co/datasets/TIGER-Lab/TheoremQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2305.12524"
COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=theoremqa
ENV EVAL_TASK_PROMPT="Solve this theorem-based problem. Print only the final answer (a number, expression, or value). Do not include any explanation or units."
