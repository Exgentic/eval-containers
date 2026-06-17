# MMLU (Massive Multitask Language Understanding)
# 14042 tasks. Data: HuggingFace cais/mmlu (all/test)
# Agent prints answer letter to stdout. Test compares to expected letter.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# `problem` reproduces the original python row: "Subject: <subject>" then a blank
# line, the question, a blank line, and the A.–D. choices (list is 1-based in
# duckdb); `answer` maps the 0-based index to its letter.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT (row_number() OVER ()-1)::VARCHAR AS id, \
       'Subject: ' || subject || E'\n\n' || question || E'\n\n' || \
       'A. ' || choices[1] || E'\n' || 'B. ' || choices[2] || E'\n' || \
       'C. ' || choices[3] || E'\n' || 'D. ' || choices[4] AS problem, \
       (['A','B','C','D'])[answer + 1] AS answer \
     FROM 'hf://datasets/cais/mmlu@c30699e8356da336a370243923dbaf21066bb9fe/all/test-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmlu"
LABEL eval.benchmark.description="MMLU - massive multitask multiple choice across 57 academic subjects"
LABEL eval.benchmark.tasks="14042"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c30699e8356da336a370243923dbaf21066bb9fe"
LABEL eval.benchmark.url="https://huggingface.co/datasets/cais/mmlu"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2009.03300"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=mmlu

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
