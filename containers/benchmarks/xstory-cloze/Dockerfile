# XStoryCloze (Cross-lingual Story Cloze)
# 16621 tasks (1511 eval examples x 11 languages). Data: HuggingFace juletxara/xstory_cloze
# Agent prints 1 or 2. Test compares to expected correct ending.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# All 11 per-language eval parquets are read in one glob; filename=true exposes
# the source path so the language code (config directory) becomes the `language`
# field. The prior python loop emitted id `<lang>_<i>` where i is the 0-based
# physical row index within each language's parquet (pyarrow read order); we
# reproduce it with file_row_number=true (the parquet's physical row position)
# rather than row_number(), since the windowed numbering does not follow the
# parquet's physical order under the glob scan. `problem` is the four input
# sentences followed by the two quiz endings, matching the old loop.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT language || '_' || file_row_number::VARCHAR AS id, \
            input_sentence_1 || E'\n' || input_sentence_2 || E'\n' || input_sentence_3 || E'\n' || input_sentence_4 || E'\n\nWhich ending is correct?\n1. ' || sentence_quiz1 || E'\n2. ' || sentence_quiz2 AS problem, \
            answer_right_ending::VARCHAR AS answer, \
            language \
     FROM ( \
       SELECT regexp_extract(filename, 'xstory_cloze@~parquet/([^/]+)/eval/', 1) AS language, * \
       FROM read_parquet('hf://datasets/juletxara/xstory_cloze@~parquet/*/eval/*.parquet', filename=true, file_row_number=true) \
     )"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xstory-cloze"
LABEL eval.benchmark.description="XStoryCloze - cross-lingual commonsense story completion across 11 languages"
LABEL eval.benchmark.tasks="16621"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c4c2d88a1ec8b37fe22166d2a610f272726724b6"
LABEL eval.benchmark.url="https://huggingface.co/datasets/juletxara/xstory_cloze"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2112.10668"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=xstory-cloze
ENV EVAL_TASK_PROMPT="Read the story and choose the correct ending. Print only the number 1 or 2, nothing else."
