# MGSM (Multilingual Grade School Math)
# 2750 tasks (250 per language x 11 languages). Data: HuggingFace juletxara/mgsm
# Agent prints answer to stdout. Test compares to expected numeric answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# All 11 per-language test parquets are read in one glob; filename=true exposes
# the source path so the language code (config directory) becomes the `language`
# field, and file_row_number=true gives the physical within-file row index.
# The prior python loop iterated a FIXED language list (bn,de,en,es,fr,ja,ru,
# sw,te,th,zh) and emitted id `<lang>_<i>` in file order. To be line-for-line
# identical the SELECT must ORDER BY that exact language order (CASE ordinal),
# then the within-file row number (the glob scan order is not preserved).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || frn::VARCHAR AS id, \
            question AS problem, \
            answer_number::VARCHAR AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename, 'mgsm@~parquet/([^/]+)/test/', 1) AS language, \
              file_row_number AS frn, * \
       FROM read_parquet('hf://datasets/juletxara/mgsm@~parquet/*/test/*.parquet', filename=true, file_row_number=true) \
     ) \
     ORDER BY CASE language WHEN 'bn' THEN 0 WHEN 'de' THEN 1 WHEN 'en' THEN 2 \
                            WHEN 'es' THEN 3 WHEN 'fr' THEN 4 WHEN 'ja' THEN 5 \
                            WHEN 'ru' THEN 6 WHEN 'sw' THEN 7 WHEN 'te' THEN 8 \
                            WHEN 'th' THEN 9 WHEN 'zh' THEN 10 END, frn"

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
ENV EVAL_BENCHMARK=mgsm
ENV EVAL_TASK_PROMPT="Solve this math problem. Print only the final numeric answer as a single number. Do not include any explanation or units."
