# syntax=docker/dockerfile:1
# LongBench v2 (THUDM, arXiv:2412.15204) — MODEL-ONLY.
#
# 503 four-choice questions (A/B/C/D) over real-world contexts of 8k to 2M words.
# Published scores come from upstream pred.py: one chat call per question with
# prompts/0shot.txt, middle-truncated to the model's window, the letter read by
# extract_answer and exact-matched. That is reproduced as a NATIVE HARNESS
# (benchmarks/RULES.md 12a) — there is no agent. /harness.sh
# (run_longbench_v2.py) truncates, calls the model once through the gateway,
# grades, and writes the reward. Pairs only with the `native` agent (12b).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives both the fetch and the label.
ARG BENCHMARK_VERSION=2b48e494f2c7a2f0af81aae178e05c7e1dde0fe9

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
# Single-JSONL task layout, root-only; the entrypoint materializes the
# requested task into /tasks/$EVAL_TASK_ID/.
COPY materialize.py /materialize.py
RUN python3 /materialize.py "${BENCHMARK_VERSION}" && chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="longbench-v2"
LABEL eval.benchmark.description="LongBench v2 - long-context multiple-choice reasoning (8k-2M words); upstream's model-only 0-shot protocol (one call, middle truncation, extract_answer exact match), no external agent"
LABEL eval.benchmark.tasks="503"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/THUDM/LongBench"
LABEL eval.benchmark.license="Apache-2.0"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2412.15204"
ENV EVAL_BENCHMARK=longbench-v2

# Upstream truncates API models' prompts with tiktoken's gpt-4o encoding; its
# BPE file is cached here (/opt/tiktoken, which /harness.sh points tiktoken at)
# so the harness runs offline.
RUN pip install --no-cache-dir tiktoken==0.14.0 \
 && TIKTOKEN_CACHE_DIR=/opt/tiktoken python3 -c "import tiktoken; tiktoken.encoding_for_model('gpt-4o-2024-08-06')" \
 && chmod -R a+rX /opt/tiktoken

COPY --from=data /tasks /tasks
COPY run_longbench_v2.py /app/run_longbench_v2.py
RUN chmod 755 /app/run_longbench_v2.py \
 && printf '#!/bin/bash\nTIKTOKEN_CACHE_DIR=/opt/tiktoken exec python3 /app/run_longbench_v2.py\n' > /harness.sh && chmod +x /harness.sh \
 && printf '#!/bin/bash\n/eval-materialize-task\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh \
 && printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt\n' > /grade.sh \
 && chmod +x /grade.sh
# The harness writes the reward itself; /grade.sh keeps a run that never reaches
# it fail-closed.
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
