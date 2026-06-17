# BFCL (Berkeley Function Calling Leaderboard)
# ~2000 tasks (simple/multiple/parallel categories). Data: HuggingFace gorilla-llm
# Agent outputs function call(s). Test compares to ground truth.
# No parquet auto-convert branch — duckdb reads the raw JSON over hf://, pinned by commit.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# categories (simple, multiple, parallel) are read in upstream order and LEFT
# JOINed to their possible_answer files by id; `id` keeps the upstream string id,
# `problem` mirrors the function defs + flattened conversation, `answer` is the
# compact JSON of ground_truth.
# Benign residual vs the old python loader (verified byte-identical after JSON
# whitespace normalization, all 800 rows, id/problem/answer): the embedded JSON
# is compact here, where python used spaced separators in `answer`
# (json.dumps) and `indent=2` pretty-printing for the function defs in
# `problem`. Same parsed JSON / same keys+values+escaping+order — only
# serializer whitespace differs. Not reproduced: matching it in SQL would need a
# fragile regex/UDF that risks corrupting string-literal contents.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "WITH src AS ( \
       SELECT 0 AS catrank, * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/BFCL_v3_simple.json',   format='newline_delimited', columns={id:'VARCHAR', question:'JSON', function:'JSON'}) \
       UNION ALL BY NAME SELECT 1 AS catrank, * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/BFCL_v3_multiple.json', format='newline_delimited', columns={id:'VARCHAR', question:'JSON', function:'JSON'}) \
       UNION ALL BY NAME SELECT 2 AS catrank, * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/BFCL_v3_parallel.json', format='newline_delimited', columns={id:'VARCHAR', question:'JSON', function:'JSON'}) ), \
     ans AS ( \
       SELECT * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/possible_answer/BFCL_v3_simple.json',   format='newline_delimited', columns={id:'VARCHAR', ground_truth:'JSON'}) \
       UNION ALL BY NAME SELECT * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/possible_answer/BFCL_v3_multiple.json', format='newline_delimited', columns={id:'VARCHAR', ground_truth:'JSON'}) \
       UNION ALL BY NAME SELECT * FROM read_json('hf://datasets/gorilla-llm/Berkeley-Function-Calling-Leaderboard@61fc0608cfd831fcfbbaa676ebdfef0ed963eeda/possible_answer/BFCL_v3_parallel.json', format='newline_delimited', columns={id:'VARCHAR', ground_truth:'JSON'}) ) \
     SELECT src.id AS id, \
       'Given these function definitions:' || chr(10) || CAST(src.function AS VARCHAR) || chr(10) || chr(10) \
         || COALESCE((SELECT string_agg((m.value ->> 'role') || ': ' || (m.value ->> 'content'), chr(10)) FROM json_each(src.question -> 0) AS m), '') AS problem, \
       CAST(to_json(COALESCE(ans.ground_truth, '[]'::JSON)) AS VARCHAR) AS answer \
     FROM src LEFT JOIN ans USING (id) \
     ORDER BY src.catrank, CAST(regexp_extract(src.id, '([0-9]+)\$', 1) AS INTEGER)"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="bfcl"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Berkeley Function Calling Leaderboard"
LABEL eval.benchmark.tasks="2000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/ShishirPatil/gorilla"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/bfcl"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bfcl"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.06840"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=bfcl
ENV EVAL_TASK_PROMPT="Call the correct function(s) based on the conversation. Print ONLY the function call(s) as JSON, nothing else."
