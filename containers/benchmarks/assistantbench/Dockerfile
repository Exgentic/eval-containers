# AssistantBench (Tel Aviv NLP)
# 33 validation tasks. Data: HuggingFace AssistantBench/AssistantBench
# Open-domain, time-consuming web research tasks. Agent browses the web and
# prints a short answer. Test compares to the reference answer (exact match).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT CAST(id AS VARCHAR) AS id, task AS problem, answer \
     FROM read_parquet('hf://datasets/AssistantBench/AssistantBench@~parquet/default/validation/0000.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assistantbench"
LABEL eval.benchmark.description="AssistantBench - realistic long-horizon web research"
LABEL eval.benchmark.tasks="33"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.data_revision="482cbbc0400f6d048438c4021727f21a10cbff49"
LABEL eval.benchmark.url="https://github.com/oriyor/assistant-bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2407.15711"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=assistantbench
ENV EVAL_TASK_PROMPT="Browse the web to answer this question. Print only the answer, nothing else."
