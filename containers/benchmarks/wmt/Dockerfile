# WMT (WMT24++)
# ~9600 tasks (~960 usable segments x 10 language pairs, en->X). Data: HuggingFace google/wmt24pp
# Agent prints translation to stdout. Test compares to expected reference translation.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# The 10 selected en->X language-pair JSONL files are read at the pinned commit
# in one pass; filename=true recovers the pair code from the source path. The
# VALUES table maps each pair to its display name (the prior python PAIRS list),
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
         ('en-fr_FR','French (France)'), ('en-de_DE','German (Germany)'), \
         ('en-es_MX','Spanish (Mexico)'), ('en-ru_RU','Russian (Russia)'), \
         ('en-zh_CN','Simplified Chinese (Mainland)'), ('en-ja_JP','Japanese (Japan)'), \
         ('en-ar_EG','Arabic (Egypt)'), ('en-hi_IN','Hindi (India)'), \
         ('en-pt_BR','Portuguese (Brazil)'), ('en-ko_KR','Korean (South Korea)') \
       ) AS t(pair, language_name) \
     ) \
     SELECT r.pair || '_' || r.segment_id::VARCHAR AS id, r.source AS problem, r.target AS answer, \
            r.pair AS language, n.language_name \
     FROM rows r JOIN names n ON n.pair = r.pair \
     WHERE NOT r.is_bad_source"

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
WORKDIR /app
ENV EVAL_BENCHMARK=wmt

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  LANG_NAME=$(cat /tasks/$EVAL_TASK_ID/language_name.txt)
  export TASK="Translate the following English text into ${LANG_NAME}. Print only the translated text with no explanation, quotes, or additional commentary.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
