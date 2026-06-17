# XNLI (Cross-lingual Natural Language Inference)
# 75150 tasks (5010 test examples x 15 languages). Data: HuggingFace facebook/xnli
# Agent prints entailment/neutral/contradiction. Test compares to expected label.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# The 15 per-language test parquets are read as an explicit path list; filename=true
# exposes the source path so the language code (parent directory) becomes the
# `language` field. (The repo also carries an `all_languages/` dir whose
# `premise`/`hypothesis` columns are STRUCTs, one field per language; a `*/` glob
# pulls it in and duckdb then fails to cast the per-language VARCHAR files into that
# struct, and the hf:// filesystem only globs `*` — so the languages are enumerated
# explicitly instead.) The prior python loop emitted id `<lang>_<i>` (i = 0-based
# per-language row index in physical file order). file_row_number=true exposes that
# exact physical position; a window row_number() instead numbers duckdb's parallel
# scan order, which permutes the parquet row groups and renumbers every id — so the
# physical file_row_number is used to reproduce `<lang>_<i>` byte-for-byte. Each
# language is a single file, so file_row_number resets to 0 per language, matching
# the per-language `i`; label 0/1/2 maps to the NLI classes.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || file_row_number::VARCHAR AS id, \
            'Premise: ' || premise || E'\nHypothesis: ' || hypothesis AS problem, \
            CASE label WHEN 0 THEN 'entailment' WHEN 1 THEN 'neutral' WHEN 2 THEN 'contradiction' END AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename, '/([^/]+)/test-00000-of-00001\.parquet\$', 1) AS language, * \
       FROM read_parquet([ \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/ar/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/bg/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/de/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/el/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/en/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/es/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/fr/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/hi/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/ru/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/sw/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/th/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/tr/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/ur/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/vi/test-00000-of-00001.parquet', \
         'hf://datasets/facebook/xnli@b8dd5d7af51114dbda02c0e3f6133f332186418e/zh/test-00000-of-00001.parquet' \
       ], filename=true, file_row_number=true) \
     )"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xnli"
LABEL eval.benchmark.description="XNLI - cross-lingual natural language inference across 15 languages"
LABEL eval.benchmark.tasks="75150"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="b8dd5d7af51114dbda02c0e3f6133f332186418e"
LABEL eval.benchmark.url="https://huggingface.co/datasets/facebook/xnli"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1809.05053"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=xnli
ENV EVAL_TASK_PROMPT="Given the premise and hypothesis, determine the relationship. Print only one word: entailment, neutral, or contradiction. Do not include any explanation."
