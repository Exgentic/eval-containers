# OpenBookQA (allenai/openbookqa)
# 500 tasks. Data: HuggingFace allenai/openbookqa (main/test split)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# `problem` rebuilds the python row: the question stem, a blank line, then one
# "label. text" line per choice (choices is a struct of parallel text/label arrays).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT id, question_stem || chr(10) || chr(10) || array_to_string( \
       list_transform(list_zip(choices.label, choices.text), lambda p: p[1] || (chr(46)||chr(32)) || p[2]), \
       chr(10)) AS problem, answerKey::VARCHAR AS answer \
     FROM 'hf://datasets/allenai/openbookqa@388097ea7776314e93a529163e0fea805b8a6454/main/test-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="openbookqa"
LABEL eval.benchmark.description="OpenBookQA - elementary science multiple choice"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="388097ea7776314e93a529163e0fea805b8a6454"
LABEL eval.benchmark.url="https://allenai.org/data/open-book-qa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1809.02789"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=openbookqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice science question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
