# OlympiadBench (text-only English math + physics)
# 910 tasks. Data: HuggingFace Hothan/OlympiadBench (OE_TO_*_en_COMP configs)
# Agent prints answer to stdout. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# Two text-only English configs are unioned. `id` keeps the original
# "<config>_<id>" form. `final_answer` is a native varchar[]; the prior python
# joined its (stripped) elements with " ; " — array_to_string reproduces that
# exactly (single-element lists collapse to the element; the few multi-answer
# physics rows join). `problem`/`subject` map straight through.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT 'OE_TO_maths_en_COMP_'||id::VARCHAR AS id, question AS problem, array_to_string(final_answer, ' ; ') AS answer, subject FROM 'hf://datasets/Hothan/OlympiadBench@~parquet/OE_TO_maths_en_COMP/train/*.parquet' \
     UNION ALL \
     SELECT 'OE_TO_physics_en_COMP_'||id::VARCHAR AS id, question AS problem, array_to_string(final_answer, ' ; ') AS answer, subject FROM 'hf://datasets/Hothan/OlympiadBench@~parquet/OE_TO_physics_en_COMP/train/*.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="olympiad-bench"
LABEL eval.benchmark.description="OlympiadBench - olympiad-level math and physics problems (text-only English split)"
LABEL eval.benchmark.tasks="910"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="91184b52131e7fc9455fef848035173aea8cc01a"
LABEL eval.benchmark.url="https://huggingface.co/datasets/Hothan/OlympiadBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2402.14008"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=olympiad-bench

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this olympiad problem. Print only the final answer in its simplest form (a number, expression, or semicolon-separated list for multi-part answers). Do not include any explanation or units.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
