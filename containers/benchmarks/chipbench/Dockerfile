# syntax=docker/dockerfile:1
# ChipBench — AI-aided chip design benchmark (UCSD/Columbia, arXiv:2601.21448).
# Model-only, native harness. Upstream: github.com/zhongkaiyu/ChipBench (MIT),
# built on NVIDIA's VerilogEval v2 harness.
#
# Scope: 223 single-turn tasks = Verilog generation (45: 30 self-contained, 6
# hierarchical, 9 CPU IP) + Verilog debugging (89 injected bugs: arithmetic /
# assignment / state machine / timing) in both upstream modes, zero-shot (spec +
# buggy code) and one-shot (+ the simulation waveform, inline). The reference-
# model track (Python/SystemC/CXXRTL) is not wired into upstream's harness at
# the pinned commit and is out of scope.
#
# Native harness (benchmarks/RULES.md 12a): upstream's evaluation is one model
# call per problem with its own prompt and extractor (scripts/sv-generate), so
# there is no agent. /harness.sh (run_chipbench.py) builds that prompt, calls the
# model once through the gateway, extracts the TopModule, and grades it with
# upstream's iverilog flow + sv-iv-analyze verdict (grade_sv.py). Shared-env.
ARG BENCHMARK_VERSION=b4ba3a06759804ec88c26c3cf15b28b9a90b3443

# Icarus Verilog v12, as upstream's Dockerfile (it builds the v12-branch; we pin
# the v12_0 release tag).
FROM debian:12-slim AS iverilog
ARG IVERILOG_REF=v12_0
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates git build-essential autoconf flex bison gperf && \
    git clone --depth 1 --branch ${IVERILOG_REF} https://github.com/steveicarus/iverilog.git /src && \
    cd /src && sh ./autoconf.sh && ./configure --prefix=/opt/iverilog && make -j"$(nproc)" && make install

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
COPY materialize.py /materialize.py
RUN python3 -c "import urllib.request as u; u.urlretrieve('https://codeload.github.com/zhongkaiyu/ChipBench/tar.gz/${BENCHMARK_VERSION}', '/tmp/cb.tgz')" && \
    mkdir /src && tar -xzf /tmp/cb.tgz -C /src --strip-components=1 && \
    python3 /materialize.py /src /tasks

FROM python:3.12-slim
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="chipbench"
LABEL eval.benchmark.description="ChipBench - Verilog generation (incl. hierarchical + CPU IP modules) and debugging of injected bugs (zero-shot and with waveform), one model call each with upstream's VerilogEval prompt, graded by iverilog simulation against the golden module. Model-only."
LABEL eval.benchmark.tasks="223"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
ARG EVAL_INTERNET=false
LABEL eval.benchmark.internet="${EVAL_INTERNET}"
ENV EVAL_INTERNET=${EVAL_INTERNET}
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/zhongkaiyu/ChipBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2601.21448"
LABEL eval.benchmark.license="MIT"
ENV EVAL_BENCHMARK=chipbench

COPY --from=iverilog /opt/iverilog /opt/iverilog
ENV PATH="/opt/iverilog/bin:${PATH}"

COPY --from=data /tasks /tasks
RUN chmod -R 600 /tasks && find /tasks -type d -exec chmod 700 {} + && chmod 711 /tasks

WORKDIR /app
COPY run_chipbench.py grade_sv.py /app/
RUN printf '#!/bin/bash\nexec python3 /app/run_chipbench.py\n' > /harness.sh && chmod +x /harness.sh && \
    printf '#!/bin/bash\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
# The harness writes the reward itself; this baseline grader keeps a run that
# never reaches it fail-closed.
RUN printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ -s /logs/verifier/reward.txt ] || echo 0.0 > /logs/verifier/reward.txt\n' > /grade.sh && \
    chmod +x /grade.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
