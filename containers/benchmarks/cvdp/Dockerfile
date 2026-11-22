# syntax=docker/dockerfile:1
# CVDP — Comprehensive Verilog Design Problems (NVIDIA, arXiv:2506.14074),
# SINGLE-TURN variant (sibling of the agent-driven `cvdp-agentic`). Model-only.
# Upstream: github.com/NVlabs/cvdp_benchmark. Dataset: nvidia/cvdp-benchmark-dataset.
#
# Scope: the 302 tasks of the v1.1.0 *non-agentic, no-commercial* code-generation
# set — RTL completion (cid002), spec→RTL (cid003), RTL modification (cid004),
# lint/QoR improvement (cid007, partly graded by Yosys synthesis), debugging
# (cid016). Verification tasks (cid012/013/014) need Cadence Xcelium and the
# comprehension tasks (cid006/008-010) are BLEU/LLM-judge scored — both out of
# scope.
#
# Native harness (benchmarks/RULES.md 12a): upstream's non-agentic mode is one
# model call per task with its own prompt and response parser, so there is no
# agent. /harness.sh (run_cvdp.py) builds upstream's exact system + user prompt,
# calls the model once through the gateway, writes the files it returns into
# /code, and runs the task's own hidden harness (core/cvdp-sim/cvdp_grade.py).
# Shared-env: every task uses the same simulator image, EVAL_TASK_ID selects one.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives both the fetch and the label.
ARG BENCHMARK_VERSION=5b807d945f6a99aa645f7e43a64a2115e281b4bf

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
ARG CVDP_SPLIT=cvdp_v1.1.0_nonagentic_code_generation_no_commercial
COPY materialize.py /materialize.py
RUN python3 -c "import urllib.request as u; u.urlretrieve('https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset/resolve/${BENCHMARK_VERSION}/${CVDP_SPLIT}.jsonl', '/tmp/split.jsonl')" && \
    python3 /materialize.py /tmp/split.jsonl /tasks

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}cvdp-sim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="cvdp"
LABEL eval.benchmark.description="CVDP single-turn - Verilog design/debug/lint tasks answered in one model call with upstream's own prompt, graded by CVDP's own cocotb/Yosys harnesses (open-source tools). Model-only."
LABEL eval.benchmark.tasks="302"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
ARG EVAL_INTERNET=false
LABEL eval.benchmark.internet="${EVAL_INTERNET}"
ENV EVAL_INTERNET=${EVAL_INTERNET}
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/NVlabs/cvdp_benchmark"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2506.14074"
ENV EVAL_BENCHMARK=cvdp

COPY --from=data /tasks /tasks
RUN chmod -R 600 /tasks && find /tasks -type d -exec chmod 700 {} + && chmod 711 /tasks

COPY run_cvdp.py /app/run_cvdp.py
RUN chmod 755 /app/run_cvdp.py && \
    printf '#!/bin/bash\nexec python3 /app/run_cvdp.py\n' > /harness.sh && chmod +x /harness.sh && \
    printf '#!/bin/bash\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
# The harness writes the reward itself; this baseline grader keeps a run that
# never reaches it fail-closed.
RUN printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ -s /logs/verifier/reward.txt ] || echo 0.0 > /logs/verifier/reward.txt\n' > /grade.sh && \
    chmod +x /grade.sh
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
