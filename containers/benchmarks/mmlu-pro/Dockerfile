# MMLU-Pro (TIGER-Lab/MMLU-Pro)
# 12032 tasks. Data: HuggingFace TIGER-Lab/MMLU-Pro
# Agent prints answer to stdout. Test compares stdout to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# `problem` reproduces the python layout: the question, then one lettered choice
# (A., B., …) per `options` entry, joined by newlines.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT question_id::VARCHAR AS id, \
            question || chr(10) || chr(10) || array_to_string(list_transform(options, (opt, j) -> chr((64 + j)::INTEGER) || '. ' || opt), chr(10)) AS problem, \
            answer \
     FROM 'hf://datasets/TIGER-Lab/MMLU-Pro@~parquet/default/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmlu-pro"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MMLU-Pro - graduate-level knowledge, multiple choice"
LABEL eval.benchmark.tasks="12032"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="54611cde22c74cca43dd78732198de6abe971398"
LABEL eval.benchmark.url="https://github.com/TIGER-AI-Lab/MMLU-Pro"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mmlu_pro"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.01574"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=mmlu-pro
ENV EVAL_TASK_PROMPT="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, D, E, F, G, H, I, or J), nothing else."
