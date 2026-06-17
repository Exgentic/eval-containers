# MATH (Hendrycks competition mathematics)
# 5000 tasks (7 subjects x test split). Data: HuggingFace EleutherAI/hendrycks_math
# Agent prints answer to stdout. Test compares to boxed expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# All 7 subject parquet files are read in one glob; filename=true exposes the
# source path so the subject (parent directory) becomes the `subject` field, and
# file_row_number=true gives the in-file physical row index (the python loader's
# `for i in range(len(t))`) so the `<subject>_<i>` id matches byte-for-byte.
# `answer` reproduces the python extract_boxed:
# the tail after the LAST `\boxed{` is split into characters whose running brace
# balance (starting at 1, inside the box) locates the matching close, yielding the
# balanced content even when nested (≈31% of rows nest braces); rows without a box
# fall back to the trimmed solution.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "WITH base AS ( \
       SELECT regexp_extract(filename, 'parquet/([^/]+)/test/', 1) AS subject, \
              file_row_number AS i, \
              problem, solution, \
              CASE WHEN solution LIKE '%\boxed{%' \
                   THEN substr(solution, length(solution) - length(reverse(split_part(reverse(solution), reverse('\boxed{'), 1))) + 1) \
                   ELSE NULL END AS tail \
       FROM read_parquet('hf://datasets/EleutherAI/hendrycks_math@~parquet/*/test/*.parquet', filename=true, file_row_number=true) \
     ), \
     chars AS ( \
       SELECT subject, i, problem, tail, \
              generate_subscripts(string_to_array(tail, ''), 1) AS pos, \
              unnest(string_to_array(tail, '')) AS ch \
       FROM base WHERE tail IS NOT NULL \
     ), \
     bal AS ( \
       SELECT subject, i, problem, tail, pos, \
              1 + sum(CASE WHEN ch='{' THEN 1 WHEN ch='}' THEN -1 ELSE 0 END) \
                  OVER (PARTITION BY subject, i ORDER BY pos ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS depth \
       FROM chars \
     ), \
     closed AS ( \
       SELECT subject, i, any_value(problem) AS problem, any_value(tail) AS tail, \
              min(pos) FILTER (WHERE depth=0) AS close_pos \
       FROM bal GROUP BY subject, i \
     ) \
     SELECT subject || '_' || i::VARCHAR AS id, problem, \
            trim(CASE WHEN close_pos IS NULL THEN tail ELSE substr(tail, 1, close_pos-1) END) AS answer, \
            subject \
     FROM closed \
     UNION ALL \
     SELECT subject || '_' || i::VARCHAR AS id, problem, trim(solution) AS answer, subject \
     FROM base WHERE tail IS NULL"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="math"
LABEL eval.benchmark.description="MATH - competition mathematics problems across 7 subjects (Hendrycks et al.)"
LABEL eval.benchmark.tasks="5000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="21a5633873b6a120296cce3e2df9d5550074f4a3"
LABEL eval.benchmark.url="https://huggingface.co/datasets/EleutherAI/hendrycks_math"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2103.03874"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=math
ENV EVAL_TASK_PROMPT="Solve this math problem. Print only the final answer in its simplest form (a number, fraction, or LaTeX expression as it would appear inside \\boxed{}). Do not include any explanation."
