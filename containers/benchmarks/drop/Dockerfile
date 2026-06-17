# DROP (Discrete Reasoning Over Paragraphs)
# 9535 tasks. Data: HuggingFace ucinlp/drop (validation)
# Agent prints answer to stdout. Test compares to first reference answer span.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# answers_spans is struct(spans varchar[], types varchar[]); answer = first span
# (duckdb lists are 1-indexed), coalesced to '' for the (currently empty) no-span set.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT query_id AS id, \
            'Passage:' || chr(10) || passage || chr(10) || chr(10) || 'Question: ' || question AS problem, \
            coalesce(answers_spans.spans[1], '') AS answer \
     FROM 'hf://datasets/ucinlp/drop@95cda593fae71b60b5b19f82de3fcf3298c1239c/data/validation-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="drop"
LABEL eval.benchmark.description="DROP - reading comprehension requiring discrete reasoning (counting, arithmetic, sorting) over paragraphs"
LABEL eval.benchmark.tasks="9535"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="95cda593fae71b60b5b19f82de3fcf3298c1239c"
LABEL eval.benchmark.url="https://huggingface.co/datasets/ucinlp/drop"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1903.00161"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=drop
ENV EVAL_TASK_PROMPT="Read the passage and answer the question. Print only the final answer with no explanation. For numeric answers, print only the number. For span answers, print the exact text from the passage."
