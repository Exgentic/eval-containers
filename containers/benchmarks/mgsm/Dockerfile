# MGSM (Multilingual Grade School Math)
# 2750 tasks (250 per language x 11 languages). Data: HuggingFace juletxara/mgsm
# Agent prints answer to stdout. Test compares to expected numeric answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# All 11 per-language test parquets are read in one glob; filename=true exposes
# the source path so the language code (config directory) becomes the `language`
# field. The prior python loop emitted id `<lang>_<i>` where i is the 0-based
# per-language row index, so row_number() partitioned by language (natural scan
# order) reproduces the same `<lang>_<i>` ids and field values.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || (row_number() OVER (PARTITION BY language) - 1)::VARCHAR AS id, \
            question AS problem, \
            answer_number::VARCHAR AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename, 'mgsm@~parquet/([^/]+)/test/', 1) AS language, * \
       FROM read_parquet('hf://datasets/juletxara/mgsm@~parquet/*/test/*.parquet', filename=true) \
     )"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mgsm"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MGSM - multilingual grade school math reasoning across 11 languages"
LABEL eval.benchmark.tasks="2750"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="8b764d53dba2de1f684b836bb45c7ce389900fde"
LABEL eval.benchmark.url="https://huggingface.co/datasets/juletxara/mgsm"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mgsm"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2210.03057"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=mgsm

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve this math problem. Print only the final numeric answer as a single number. Do not include any explanation or units.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
