# HellaSwag
# 10042 tasks. Data: HuggingFace Rowan/hellaswag (validation)
# Agent prints the letter of the most plausible continuation. Test compares letters.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# problem string mirrors the original loader: "Activity:", blank line, "Context:",
# blank line, the prompt, then one "<letter>. <ending>" line per choice (the four
# endings, zipped with A-D and joined). answer = letter at the label index; id = ind.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT ind::VARCHAR AS id, \
       'Activity: ' || activity_label || chr(10) || chr(10) || \
       'Context: ' || ctx || chr(10) || chr(10) || \
       'Which ending is most likely?' || chr(10) || \
       array_to_string(list_transform(list_zip(['A','B','C','D'], endings), \
         lambda p: p[1] || '. ' || p[2]), chr(10)) AS problem, \
       ['A','B','C','D'][label::INTEGER + 1] AS answer \
     FROM 'hf://datasets/Rowan/hellaswag@218ec52e09a7e7462a5400043bb9a69a41d06b76/data/validation-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="hellaswag"
LABEL eval.benchmark.description="HellaSwag - commonsense sentence completion: pick the most plausible ending"
LABEL eval.benchmark.tasks="10042"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="218ec52e09a7e7462a5400043bb9a69a41d06b76"
LABEL eval.benchmark.url="https://huggingface.co/datasets/Rowan/hellaswag"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1905.07830"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=hellaswag
ENV EVAL_TASK_PROMPT="Pick the most plausible continuation of the context. Print only the letter of the correct answer (A, B, C, or D), nothing else."
