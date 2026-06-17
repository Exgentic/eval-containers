# MedMCQA (openlifescienceai/medmcqa)
# 4183 tasks. Data: HuggingFace openlifescienceai/medmcqa (validation split - test has no labels)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT id::VARCHAR AS id, \
     question || E'\n\n' || 'A. ' || opa || E'\n' || 'B. ' || opb || E'\n' || 'C. ' || opc || E'\n' || 'D. ' || opd AS problem, \
     (['A','B','C','D'])[cop + 1] AS answer \
     FROM 'hf://datasets/openlifescienceai/medmcqa@91c6572c454088bf71b679ad90aa8dffcd0d5868/data/validation-00000-of-00001.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="medmcqa"
LABEL eval.benchmark.description="MedMCQA - multi-subject multi-choice medical entrance exam questions"
LABEL eval.benchmark.tasks="4183"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="91c6572c454088bf71b679ad90aa8dffcd0d5868"
LABEL eval.benchmark.url="https://github.com/MedMCQA/MedMCQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2203.14371"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=medmcqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this medical multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
