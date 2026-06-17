# GSM8K (Grade School Math 8K)
# 1319 tasks. Data: HuggingFace openai/gsm8k (main/test)
# Agent prints answer to stdout. Test compares to expected numeric answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# `answer` keeps only the text after the `####` marker, trimmed, with thousands
# separators stripped (mirrors the prior python split('####')[-1].replace(',','')).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, question AS problem, \
            replace(trim(split_part(answer, '####', 2)), ',', '') AS answer \
     FROM 'hf://datasets/openai/gsm8k@~parquet/main/test/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="gsm8k"
LABEL eval.benchmark.description="GSM8K - grade school math word problems requiring multi-step reasoning"
LABEL eval.benchmark.tasks="1319"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="740312add88f781978c0658806c59bc2815b9866"
LABEL eval.benchmark.url="https://huggingface.co/datasets/openai/gsm8k"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2110.14168"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=gsm8k
ENV EVAL_TASK_PROMPT="Solve this grade school math problem. Print only the final numeric answer as a single number with no units, commas, or explanation."
