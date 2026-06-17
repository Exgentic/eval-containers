# ACPBench
# 7 reasoning tasks x 13 domains. Data: HuggingFace ibm-research/acp_bench (MCQ split).
# Agent prints a single letter (A/B/C/D). Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# splits (boolean/generative are graded differently and deferred). Each branch reads
# one config's test parquet; row_number() is per-branch, giving the original
# per-group "<group>_<i>" ids. problem = context + blank line + question.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT 'acp_app_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_app_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_app_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_areach_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_areach_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_areach_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_just_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_just_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_just_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_land_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_land_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_land_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_prog_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_prog_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_prog_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_reach_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_reach_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_reach_mcq/test/*.parquet' \
     UNION ALL \
     SELECT 'acp_val_mcq_'||(row_number() OVER ()-1)::VARCHAR AS id, context||chr(10)||chr(10)||question AS problem, trim(answer) AS answer, 'acp_val_mcq' AS \"group\" FROM 'hf://datasets/ibm-research/acp_bench@~parquet/acp_val_mcq/test/*.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="acpbench"
LABEL eval.benchmark.description="ACPBench - Action, Change, and Planning reasoning benchmark"
LABEL eval.benchmark.tasks="1040"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="05e5883d9afbcfa3bdc8f270cb345ef0b9526d4a"
LABEL eval.benchmark.url="https://ibm.github.io/ACPBench/"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2410.05669"
COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=acpbench
ENV EVAL_TASK_PROMPT="Answer this multiple choice question. Print only the letter (A, B, C, or D), nothing else."
