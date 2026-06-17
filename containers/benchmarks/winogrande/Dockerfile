# WinoGrande (winogrande_xl validation)
# 1267 tasks. Data: HuggingFace allenai/winogrande (winogrande_xl/validation)
# Agent prints "1" or "2" indicating which option fills the blank.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, \
            sentence || E'\n\n1. ' || option1 || E'\n2. ' || option2 AS problem, \
            answer::VARCHAR AS answer \
     FROM 'hf://datasets/allenai/winogrande@01e74176c63542e6b0bcb004dcdea22d94fb67b5/winogrande_xl/validation-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="winogrande"
LABEL eval.benchmark.description="WinoGrande - large-scale Winograd-style commonsense pronoun resolution (xl validation)"
LABEL eval.benchmark.tasks="1267"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="01e74176c63542e6b0bcb004dcdea22d94fb67b5"
LABEL eval.benchmark.url="https://huggingface.co/datasets/allenai/winogrande"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1907.10641"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=winogrande
ENV EVAL_TASK_PROMPT="The sentence has a blank marked with _. Choose which option correctly fills the blank. Print only the digit 1 or 2, nothing else."
