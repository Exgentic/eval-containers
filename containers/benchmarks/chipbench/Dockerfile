# syntax=docker/dockerfile:1
# ChipBench — AI-aided chip design benchmark (UCSD/Columbia, arXiv:2601.21448).
# Model-only, native harness. Upstream: github.com/zhongkaiyu/ChipBench (MIT),
# built on NVIDIA's VerilogEval v2 harness.
#
# Scope: 268 single-turn tasks, the paper's three tracks = Verilog generation
# (45: 30 self-contained, 6 hierarchical, 9 CPU IP) + Verilog debugging (89
# injected bugs: arithmetic / assignment / state machine / timing) in both
# upstream modes, zero-shot (spec + buggy code) and one-shot (+ the simulation
# waveform, inline) + Python reference models of the 45 generation problems
# (paper Table 3). SystemC/CXXRTL reference models are not reported in the paper
# and SystemC is not wired into upstream's checker; both are out of scope.
#
# Native harness (benchmarks/RULES.md 12a): upstream's evaluation is one model
# call per problem with its own prompt and extractor (scripts/sv-generate), so
# there is no agent. /harness.sh (run_chipbench.py) builds that prompt, calls the
# model once through the gateway, extracts the TopModule, and grades it with
# upstream's iverilog flow + sv-iv-analyze verdict (grade_sv.py); reference
# models are graded by upstream's Tool_Box/crosslang_verify (grade_py.py).
# Shared-env. The toolchain is upstream's Dockerfile: Ubuntu 22.04 with its
# packaged Verilator (4.038) and Python 3.11, Icarus Verilog built from source.
ARG BENCHMARK_VERSION=b4ba3a06759804ec88c26c3cf15b28b9a90b3443
# The checker is code, not task data: pinned on its own so a build at an older
# BENCHMARK_VERSION (the paper's 2dfbb80 predates this checker) still grades
# reference models with it.
ARG TOOLBOX_REF=b4ba3a06759804ec88c26c3cf15b28b9a90b3443

# Icarus Verilog v12, as upstream's Dockerfile (it builds the v12-branch; we pin
# the v12_0 release tag).
FROM ubuntu:22.04 AS iverilog
ARG IVERILOG_REF=v12_0
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      ca-certificates git build-essential autoconf flex bison gperf && \
    git clone --depth 1 --branch ${IVERILOG_REF} https://github.com/steveicarus/iverilog.git /src && \
    cd /src && sh ./autoconf.sh && ./configure --prefix=/opt/iverilog && make -j"$(nproc)" && make install

FROM python:3.12-slim AS data
ARG BENCHMARK_VERSION
ARG TOOLBOX_REF
COPY materialize.py /materialize.py
RUN python3 -c "import urllib.request as u; u.urlretrieve('https://codeload.github.com/zhongkaiyu/ChipBench/tar.gz/${BENCHMARK_VERSION}', '/tmp/cb.tgz'); u.urlretrieve('https://codeload.github.com/zhongkaiyu/ChipBench/tar.gz/${TOOLBOX_REF}', '/tmp/tb.tgz')" && \
    mkdir /src /tb && tar -xzf /tmp/cb.tgz -C /src --strip-components=1 && \
    tar -xzf /tmp/tb.tgz -C /tb --strip-components=1 && \
    python3 /materialize.py /src /tasks && \
    cp -r /tb/Tool_Box/crosslang_verify /crosslang_verify && \
    find /crosslang_verify -name __pycache__ -prune -exec rm -rf {} +
# One deviation, closing a hole: the checker's Python comparison skips any output
# missing from the model's returned dict, so a model reporting nothing passed.
# Patched in the C++ testbench it generates, outside the answer's process: a
# missing output counts as a mismatch.
RUN python3 - <<'PY'
p = "/crosslang_verify/tools/dut.py"
src = open(p).read()
old = "                if (it != py_outputs[i].end()) {{\n"
new = """                if (it == py_outputs[i].end()) {{
                    if ({ev} < 10) {{
                        std::cerr << "[{self.label} MISMATCH] {label} " << (i - WARMUP_CYCLES) << ", " << name
                                  << ": expected=" << expected << ", got=<missing>" << std::endl;
                    }}
                    {ev}++;
                }} else {{
"""
assert src.count(old) == 1, "crosslang_verify's Python comparison changed"
open(p, "w").write(src.replace(old, new))
PY

FROM ubuntu:22.04
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="chipbench"
LABEL eval.benchmark.description="ChipBench - Verilog generation (incl. hierarchical + CPU IP modules), debugging of injected bugs (zero-shot and with waveform) and Python reference models, one model call each with upstream's prompts, graded by simulation against the golden module (iverilog; Verilator for reference models). Model-only."
LABEL eval.benchmark.tasks="268"
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

# Upstream's runtime: Python 3.11, Verilator + a C++ toolchain for the checker.
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      python3.11 verilator perl g++ make && \
    update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.11 1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=iverilog /opt/iverilog /opt/iverilog
ENV PATH="/opt/iverilog/bin:${PATH}"
COPY --from=data /crosslang_verify /opt/crosslang_verify

COPY --from=data /tasks /tasks
RUN chmod -R 600 /tasks && find /tasks -type d -exec chmod 700 {} + && chmod 711 /tasks

WORKDIR /app
COPY run_chipbench.py grade_sv.py grade_py.py /app/
RUN printf '#!/bin/bash\nexec python3 /app/run_chipbench.py\n' > /harness.sh && chmod +x /harness.sh && \
    printf '#!/bin/bash\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
# The harness writes the reward itself; this baseline grader keeps a run that
# never reaches it fail-closed.
RUN printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ -s /logs/verifier/reward.txt ] || echo 0.0 > /logs/verifier/reward.txt\n' > /grade.sh && \
    chmod +x /grade.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
