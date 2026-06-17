# XCOPA (Cross-lingual Choice of Plausible Alternatives)
# 5500 tasks (500 test examples x 11 languages). Data: HuggingFace cambridgeltl/xcopa
# Agent prints A or B. Test compares to expected choice.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# languages live in per-language parquet dirs (the dataset also ships translation-*
# configs — excluded by the IN list); the path encodes the language, the in-file
# idx column (0..499) reproduces the original per-language row index.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || idx::VARCHAR AS id, \
            premise || chr(10) || (CASE WHEN question='cause' THEN 'What was the cause?' ELSE 'What was the effect?' END) \
              || chr(10) || 'A. ' || choice1 || chr(10) || 'B. ' || choice2 AS problem, \
            CASE WHEN label=0 THEN 'A' ELSE 'B' END AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename,'parquet/([a-z]+)/test',1) AS language, * \
       FROM read_parquet('hf://datasets/cambridgeltl/xcopa@~parquet/*/test/*.parquet', filename=true) \
       WHERE regexp_extract(filename,'parquet/([a-z]+)/test',1) IN ('et','ht','id','it','qu','sw','ta','th','tr','vi','zh') \
     )"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xcopa"
LABEL eval.benchmark.description="XCOPA - cross-lingual causal commonsense reasoning across 11 languages"
LABEL eval.benchmark.tasks="5500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="042f78955ba48e6404616762fa6e05e839c3907a"
LABEL eval.benchmark.url="https://huggingface.co/datasets/cambridgeltl/xcopa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2005.00333"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=xcopa
ENV EVAL_TASK_PROMPT="Choose the more plausible alternative. Print only the letter A or B, nothing else."
