# CommonsenseQA (tau/commonsense_qa)
# 1221 tasks. Data: HuggingFace tau/commonsense_qa (validation split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# problem string mirrors the original loader: question, a blank line, then one
# "<label>. <text>" line per choice (the choices struct holds parallel label/text
# arrays, zipped and joined).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT id, question || chr(10) || chr(10) || array_to_string( \
       list_transform(list_zip(choices.label, choices.text), lambda p: p[1] || '. ' || p[2]), \
       chr(10)) AS problem, answerKey AS answer \
     FROM 'hf://datasets/tau/commonsense_qa@94630fe30dad47192a8546eb75f094926d47e155/data/validation-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="commonsenseqa"
LABEL eval.benchmark.description="CommonsenseQA - commonsense multiple choice QA"
LABEL eval.benchmark.tasks="1221"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="94630fe30dad47192a8546eb75f094926d47e155"
LABEL eval.benchmark.url="https://www.tau-nlp.sites.tau.ac.il/commonsenseqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1811.00937"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=commonsenseqa
ENV EVAL_TASK_PROMPT="Answer this commonsense multiple choice question. Print only the letter of the correct answer (A, B, C, D, or E), nothing else."
