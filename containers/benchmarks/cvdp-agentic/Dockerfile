# syntax=docker/dockerfile:1
# CVDP — Comprehensive Verilog Design Problems (NVIDIA, arXiv:2506.14074), AGENTIC
# variant (sibling of the single-turn, native-harness `cvdp`).
# Upstream: github.com/NVlabs/cvdp_benchmark. Dataset: nvidia/cvdp-benchmark-dataset.
#
# Scope: the 92 tasks of the v1.1.0 *agentic, no-commercial* code-generation set
# (spec→RTL cid003, RTL modification cid004, module reuse cid005, debugging
# cid016). Every verification task (cid012/013/014) needs Cadence Xcelium and
# lives only in the *commercial* split, out of scope here.
#
# Shared-env: every task uses the same simulator image (core/cvdp-sim, upstream's
# __OSS_SIM_IMAGE__ rebuilt from its own recipe), so ONE image holds all 92 tasks
# and EVAL_TASK_ID selects one at run time. An external agent (Claude Code, …)
# edits upstream's own /code/{docs,rtl,verif} workspace; grading runs the task's
# own hidden CVDP harness, unmodified (core/cvdp-sim/cvdp_grade.py).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives both the fetch and the label.
ARG BENCHMARK_VERSION=5b807d945f6a99aa645f7e43a64a2115e281b4bf

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
ARG CVDP_SPLIT=cvdp_v1.1.0_agentic_code_generation_no_commercial
COPY materialize.py /materialize.py
RUN python3 -c "import urllib.request as u; u.urlretrieve('https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset/resolve/${BENCHMARK_VERSION}/${CVDP_SPLIT}.jsonl', '/tmp/split.jsonl')" && \
    python3 /materialize.py /tmp/split.jsonl /tasks

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}cvdp-sim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="cvdp-agentic"
LABEL eval.benchmark.description="CVDP agentic - Verilog design/debug tasks an agent solves in a workspace, graded by CVDP's own cocotb simulation harnesses (open-source tools)"
LABEL eval.benchmark.tasks="92"
LABEL eval.benchmark.env="shared-env"
# Simulation is offline; a web fetch could only find upstream sources.
ARG EVAL_INTERNET=false
LABEL eval.benchmark.internet="${EVAL_INTERNET}"
ENV EVAL_INTERNET=${EVAL_INTERNET}
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/NVlabs/cvdp_benchmark"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2506.14074"
ENV EVAL_BENCHMARK=cvdp-agentic

COPY --from=data /tasks /tasks
# /tasks is the task store: root-only. 711 on the dir so the agent cannot list the
# task ids (rule 7) — and the harness + any reference fields stay unreadable.
RUN chmod -R 600 /tasks && find /tasks -type d -exec chmod 700 {} + && chmod 711 /tasks

COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
