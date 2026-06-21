# MedQA (GBaker/MedQA-USMLE-4-options)
# 1273 tasks. Data: HuggingFace GBaker/MedQA-USMLE-4-options (test split)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22). id is the physical row
# index (file_row_number); options is a struct of A/B/C/D choice strings.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT file_row_number::VARCHAR AS id, \
            question || chr(10)||chr(10) || \
            'A. '||options.A||chr(10)||'B. '||options.B||chr(10)|| \
            'C. '||options.C||chr(10)||'D. '||options.D AS problem, \
            answer_idx AS answer \
     FROM read_parquet('hf://datasets/GBaker/MedQA-USMLE-4-options@~parquet/default/test/0000.parquet', file_row_number=true)"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="medqa"
LABEL eval.benchmark.description="MedQA - USMLE-style medical licensing exam questions, 4 options"
LABEL eval.benchmark.tasks="1273"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18"
LABEL eval.benchmark.url="https://github.com/jind11/MedQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2009.13081"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=medqa
ENV EVAL_TASK_PROMPT="Answer this USMLE-style medical question. Print only the letter of the correct answer (A, B, C, or D), nothing else."
