# WMT (WMT24++)
# ~9600 tasks (~960 usable segments x 10 language pairs, en->X). Data: HuggingFace google/wmt24pp
# Agent prints translation to stdout. Test compares to expected reference translation.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# The 10 selected en->X language-pair JSONL files are read at the pinned commit
# in one pass; filename=true recovers the pair code from the source path. The
# VALUES table maps each pair to its display name (the prior python PAIRS list)
# and an ordinal, so `ORDER BY ord, segment_id` reproduces the old loop's order
# (PAIRS list, then file rows) line-for-line — EVAL_TASK_ID selects by line.
# is_bad_source is a native boolean so `NOT is_bad_source` reproduces the old
# string-compare skip, and `<pair>_<segment_id>` matches the prior python ids.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "WITH rows AS ( \
       SELECT regexp_extract(filename, 'wmt24pp@[^/]+/([^/]+)\.jsonl', 1) AS pair, \
              segment_id, source, target, is_bad_source \
       FROM read_json_auto([ \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-fr_FR.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-de_DE.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-es_MX.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-ru_RU.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-zh_CN.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-ja_JP.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-ar_EG.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-hi_IN.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-pt_BR.jsonl', \
         'hf://datasets/google/wmt24pp@e65f5856b1de3319c748c15e5aec0bc2336ec3b0/en-ko_KR.jsonl' \
       ], filename=true) \
     ), \
     names AS ( \
       SELECT * FROM (VALUES \
         ('en-fr_FR','French (France)',1), ('en-de_DE','German (Germany)',2), \
         ('en-es_MX','Spanish (Mexico)',3), ('en-ru_RU','Russian (Russia)',4), \
         ('en-zh_CN','Simplified Chinese (Mainland)',5), ('en-ja_JP','Japanese (Japan)',6), \
         ('en-ar_EG','Arabic (Egypt)',7), ('en-hi_IN','Hindi (India)',8), \
         ('en-pt_BR','Portuguese (Brazil)',9), ('en-ko_KR','Korean (South Korea)',10) \
       ) AS t(pair, language_name, ord) \
     ) \
     SELECT r.pair || '_' || r.segment_id::VARCHAR AS id, r.source AS problem, r.target AS answer, \
            r.pair AS language, n.language_name \
     FROM rows r JOIN names n ON n.pair = r.pair \
     WHERE NOT r.is_bad_source \
     ORDER BY n.ord, r.segment_id::BIGINT"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="wmt"
LABEL eval.benchmark.description="WMT24++ - multilingual machine translation from English to 10 target locales"
LABEL eval.benchmark.tasks="9600"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="e65f5856b1de3319c748c15e5aec0bc2336ec3b0"
LABEL eval.benchmark.url="https://huggingface.co/datasets/google/wmt24pp"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.12404"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=wmt
ENV EVAL_TASK_PROMPT="Translate the following English text into {language_name}. Print only the translated text with no explanation, quotes, or additional commentary."
