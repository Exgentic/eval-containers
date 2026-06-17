# ARC (AI2 Reasoning Challenge - Challenge set)
# 1172 tasks. Data: HuggingFace allenai/ai2_arc (ARC-Challenge/test)
# Agent prints answer letter to stdout. Test compares to expected letter.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# `problem` rebuilds the python row: question, a blank line, then one "label. text"
# line per choice (choices is a struct of parallel label/text arrays).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT id::VARCHAR AS id, \
            question || chr(10) || chr(10) || array_to_string(list_transform(list_zip(choices.label, choices.text), lambda p: p[1] || (chr(46)||chr(32)) || p[2]), chr(10)) AS problem, \
            answerKey::VARCHAR AS answer \
     FROM 'hf://datasets/allenai/ai2_arc@210d026faf9955653af8916fad021475a3f00453/ARC-Challenge/test-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="arc"
LABEL eval.benchmark.description="ARC Challenge - grade-school science multiple choice questions requiring reasoning"
LABEL eval.benchmark.tasks="1172"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="210d026faf9955653af8916fad021475a3f00453"
LABEL eval.benchmark.url="https://huggingface.co/datasets/allenai/ai2_arc"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1803.05457"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=arc
ENV EVAL_TASK_PROMPT="Answer this science multiple choice question. Print only the label of the correct answer (e.g., A), nothing else."
