# Global-MMLU (CohereLabs/Global-MMLU)
# 589764 tasks (14042 test questions x 42 languages). Data: HuggingFace CohereLabs/Global-MMLU
# Agent prints letter A/B/C/D. Test compares to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# All 42 per-language parquet files are read in one glob; filename=true exposes
# the source path so the language code (parent directory) becomes the `language`
# field and the `<lang>_<sample_id>` id, matching the prior python row dict.
# COALESCE(option_*, 'None') reproduces the old python row dict, whose f-string
# rendered a NULL/None choice as the literal str(None) -> 'None'; without it the
# SQL || concat would propagate NULL and blank the whole `problem` (4 rows).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || sample_id AS id, \
            question || E'\n\nA. ' || COALESCE(option_a, 'None') || E'\nB. ' || COALESCE(option_b, 'None') || E'\nC. ' || COALESCE(option_c, 'None') || E'\nD. ' || COALESCE(option_d, 'None') AS problem, \
            upper(trim(answer)) AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename, '/([^/]+)/test-[^/]*\.parquet\$', 1) AS language, * \
       FROM read_parquet('hf://datasets/CohereLabs/Global-MMLU@0e619dbeb34206cd48705a1a0ea7fb21cae09993/*/test-*.parquet', filename=true) \
     )"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="global-mmlu"
LABEL eval.benchmark.description="Global-MMLU - multilingual knowledge and reasoning multiple choice across 42 languages"
LABEL eval.benchmark.tasks="589764"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0e619dbeb34206cd48705a1a0ea7fb21cae09993"
LABEL eval.benchmark.url="https://huggingface.co/datasets/CohereLabs/Global-MMLU"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2412.03304"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=global-mmlu
ENV EVAL_TASK_PROMPT="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else."
