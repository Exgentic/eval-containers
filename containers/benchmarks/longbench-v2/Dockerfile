# LongBench v2 - THUDM/LongBench-v2 (now hosted as zai-org/LongBench-v2)
# 503 four-choice questions (A/B/C/D) over long real-world contexts, 8k to 2M
# words. Ref: https://github.com/THUDM/LongBench (arXiv:2412.15204)
#
# Protocol (upstream pred.py, default w/o-CoT setting): the prompt is
# prompts/0shot.txt filled with the stripped context, question and choices; the
# answer is the letter extracted from "The correct answer is (X)" by upstream's
# extract_answer; the score is exact match of that letter. The rendered prompt is
# too large for an env var (up to ~16 MB), so the agent reads it from
# /app/task.txt, like longbench and mrcr.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives both the fetch and the label.
ARG BENCHMARK_VERSION=2b48e494f2c7a2f0af81aae178e05c7e1dde0fe9

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
# Single-JSONL task layout: one line per task, root-only; the entrypoint
# materializes the requested task into /tasks/$EVAL_TASK_ID/.
COPY materialize.py /materialize.py
RUN python3 /materialize.py "${BENCHMARK_VERSION}" && chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="longbench-v2"
LABEL eval.benchmark.description="LongBench v2 - long-context multiple-choice reasoning (8k-2M words), official 0-shot prompt and answer extraction"
LABEL eval.benchmark.tasks="503"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/THUDM/LongBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2412.15204"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=longbench-v2

# Upstream's answer extraction and exact-match judge; root-only (rule 5).
COPY grade.py /tests/grade.py
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod -R 700 /tests && chmod +x /grade.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
