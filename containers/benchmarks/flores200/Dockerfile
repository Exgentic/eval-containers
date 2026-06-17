# FLORES-200 (Facebook No Language Left Behind)
# 10120 tasks (1012 devtest sentences x 10 target languages, en->X). Data: NLLB FLORES-200 devtest
# Agent prints translation to stdout. Test compares to expected reference translation.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Data strategy:
# Meta pulled the anonymous dl.fbaipublicfiles.com/nllb link, so we switch to
# `openlanguagedata/flores_plus` on HuggingFace (maintained successor of the
# FLORES-200 devtest, CC-BY-SA-4.0). The main repo is license-gated, which
# means anyone with an HF account that has accepted the license can download —
# no private credential. Build requires a build-time secret `HF_TOKEN`.
# One duckdb query
# secret + retry). The dataset's `@~parquet` auto-convert branch exposes one
# config per language (eng_Latn, fra_Latn, …); each devtest row carries
# {id, text, …} where `id` is the cross-language sentence id (0-indexed,
# contiguous), so sorted position == id. We read the 11 needed configs
# (source + 10 targets) in one pass (filename=true recovers the config code),
# then for each target join the English source to the target on `id` — the
# python-free equivalent of the old "sort each language by id, zip by
# position" alignment. Row order follows the target list (ord) to keep task
# ids stable. NOTE: the parquet branch names Simplified Chinese `cmn_Hans`
# (individual code) where the legacy loader emitted `zho_Hans` (same FLORES
# devtest text); we read the `cmn_Hans` parquet (src) but emit the original
# `zho_Hans` language code/name so the JSONL fields match the legacy output.
# Benign residual (cannot byte-verify): this same rename broke the legacy
# loader upstream — it does `hf_hub_download('devtest/zho_Hans.jsonl')`, which
# now 404s (the file is `cmn_Hans.jsonl`), so the old loader crashes before
# writing /tasks and can no longer produce a ground-truth all.jsonl to diff
# against. Verified the spec is reproduced exactly: substituting cmn_Hans for
# the dead zho_Hans download in the old python yields a normalized diff of 0
# rows vs this loader's output across all 10120 rows.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "WITH base AS ( \
       SELECT regexp_extract(filename, 'flores_plus@~parquet/([^/]+)/devtest', 1) AS cfg, id, text \
       FROM read_parquet([ \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/eng_Latn/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/fra_Latn/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/spa_Latn/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/deu_Latn/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/rus_Cyrl/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/cmn_Hans/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/jpn_Jpan/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/arb_Arab/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/hin_Deva/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/swh_Latn/devtest/0000.parquet', \
         'hf://datasets/openlanguagedata/flores_plus@~parquet/por_Latn/devtest/0000.parquet' \
       ], filename=true) \
     ), \
     src AS (SELECT id, text FROM base WHERE cfg='eng_Latn'), \
     tgt AS ( \
       SELECT * FROM (VALUES \
         (1,'fra_Latn','fra_Latn','French'), (2,'spa_Latn','spa_Latn','Spanish'), \
         (3,'deu_Latn','deu_Latn','German'), (4,'rus_Cyrl','rus_Cyrl','Russian'), \
         (5,'cmn_Hans','zho_Hans','Simplified Chinese'), (6,'jpn_Jpan','jpn_Jpan','Japanese'), \
         (7,'arb_Arab','arb_Arab','Modern Standard Arabic'), (8,'hin_Deva','hin_Deva','Hindi'), \
         (9,'swh_Latn','swh_Latn','Swahili'), (10,'por_Latn','por_Latn','Portuguese') \
       ) AS t(ord, cfg, code, name) \
     ) \
     SELECT 'eng_Latn_' || tgt.code || '_' || s.id AS id, s.text AS problem, b.text AS answer, \
            tgt.code AS language, tgt.name AS language_name \
     FROM tgt JOIN base b ON b.cfg = tgt.cfg JOIN src s ON s.id = b.id \
     ORDER BY tgt.ord, s.id"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="flores200"
LABEL eval.benchmark.description="FLORES-200 - multilingual machine translation from English to 10 target languages"
LABEL eval.benchmark.tasks="10120"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="flores_plus-v2.0"
LABEL eval.benchmark.url="https://huggingface.co/datasets/openlanguagedata/flores_plus"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2207.04672"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=flores200
ENV EVAL_TASK_PROMPT="Translate the following English sentence into {language_name}. Print only the translated sentence with no explanation, quotes, or additional text."
