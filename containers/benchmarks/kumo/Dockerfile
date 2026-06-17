# Kumo (diagnostic reasoning)
# 250 tasks. Data: HuggingFace pkuHaowei/kumo-hard
# Agent observes scenario and selects actions. Test compares to ground truth.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# The dataset ships one parquet per environment; the `@~parquet` auto-convert
# branch exposes them as default/<Env>/0000.parquet, read in one glob with
# filename=true so the env name (parent dir) becomes the `<Env>_<i>` id prefix.
# `i` is the 0-based parquet row index *in file order* (file_row_number; each
# env is a single file so it equals the per-env index) — this reproduces the
# prior python `for i in range(len(t)): id=f'{env}_{i}'` byte-for-byte. The old
# loop iterated envs alphabetically, so `ORDER BY env, i` matches that order.
# `problem` concatenates the observation with the action list; `to_json` emits a
# `["a","b"]` array, and the `replace('","','", "')` re-inserts the comma-space
# that python's `json.dumps(action_list)` writes between elements (no element
# contains the literal `","` separator, so the rewrite is exact). `answer` is
# ground_truth.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT regexp_extract(filename, 'default/([^/]+)/', 1) || '_' || file_row_number::VARCHAR AS id, \
            observation || E'\n\nAvailable actions:\n' || replace(to_json(action_list), '\",\"', '\", \"') AS problem, \
            ground_truth AS answer \
     FROM read_parquet('hf://datasets/pkuHaowei/kumo-hard@~parquet/default/*/0000.parquet', filename=true, file_row_number=true) \
     ORDER BY regexp_extract(filename, 'default/([^/]+)/', 1), file_row_number"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="kumo"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Kumo - diagnostic reasoning"
LABEL eval.benchmark.tasks="250"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/Haowei-PKU/Kumo"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/kumo"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2505.08857"
COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=kumo
ENV EVAL_TASK_PROMPT="Based on the observation, select the correct action. Print ONLY the action, nothing else."
