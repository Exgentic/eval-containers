# PubMedQA (qiaojin/PubMedQA)
# 1000 tasks. Data: HuggingFace qiaojin/PubMedQA (pqa_labeled/train)
# Agent reads a biomedical abstract and answers yes/no/maybe to a question.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22). context is a struct of
# parallel contexts[]/labels[] lists; zip them into "label: paragraph" blocks.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT pubid::VARCHAR AS id, \
            'Abstract:' || chr(10) || \
            array_to_string(list_transform(context.contexts, \
                (p, j) -> CASE WHEN j <= len(context.labels) AND context.labels[j] <> '' \
                               THEN context.labels[j] || ': ' || p ELSE p END), chr(10)||chr(10)) || \
            chr(10)||chr(10) || 'Question: ' || question AS problem, \
            final_decision AS answer \
     FROM read_parquet('hf://datasets/qiaojin/PubMedQA@~parquet/pqa_labeled/train/0000.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="pubmedqa"
LABEL eval.benchmark.description="PubMedQA - biomedical research question answering from abstracts"
LABEL eval.benchmark.tasks="1000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="9001f2853fb87cab8d220904e0de81ac6973b318"
LABEL eval.benchmark.url="https://github.com/pubmedqa/pubmedqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1909.06146"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=pubmedqa
ENV EVAL_TASK_PROMPT="Read the biomedical research abstract and answer the question. Print only one of: yes, no, or maybe. Nothing else."
