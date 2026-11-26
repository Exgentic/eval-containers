# syntax=docker/dockerfile:1
# CVDP simulator base, shared by benchmarks/cvdp (single-turn, native) and
# benchmarks/cvdp-agentic (agent-driven): upstream's open-source simulation image,
# rebuilt verbatim from its own recipe (NVlabs/cvdp_benchmark docker/Dockerfile.sim
# at the pinned commit — nvidia/cvdp-sim:v1.0.0 is not published to any registry;
# upstream's README says to build it locally), plus the shared CVDP grader.
#
# Icarus Verilog v13_0, Yosys 0.40, Verilator v5.038, and upstream's hash-locked
# Python requirements (cocotb 2.0.1, pytest 8.3.2, cocotb-bus, cocotbext-axi/pcie,
# aes, coverage) in /venv. Since v1.1.0 upstream's OSS_PNR_IMAGE defaults to this
# same image, so it serves the synthesis (yosys) harness services too.
FROM ubuntu:24.04
LABEL eval.type="core-base"
LABEL eval.base.kind="cvdp-sim"
ARG DEBIAN_FRONTEND=noninteractive
ARG CVDP_REPO_REV=8e894cf74414ab1eaea1e2b4e80a02f123df07b6
LABEL eval.base.upstream="https://github.com/NVlabs/cvdp_benchmark/blob/${CVDP_REPO_REV}/docker/Dockerfile.sim"
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential git curl ca-certificates \
    python3 python3-dev python3-pip python3-venv \
    gperf flex bison autoconf automake libtool make perl help2man \
    libfl2 libfl-dev zlib1g zlib1g-dev \
    tcl-dev libreadline-dev libffi-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 --branch v13_0 https://github.com/steveicarus/iverilog.git /tmp/iverilog && \
    cd /tmp/iverilog && sh autoconf.sh && ./configure --prefix=/usr/local && \
    make -j"$(nproc)" && make install && rm -rf /tmp/iverilog
RUN git clone --depth 1 --branch yosys-0.40 https://github.com/YosysHQ/yosys.git /tmp/yosys && \
    cd /tmp/yosys && make -j"$(nproc)" PREFIX=/usr/local && \
    make install PREFIX=/usr/local && rm -rf /tmp/yosys
RUN git clone --depth 1 --branch v5.038 https://github.com/verilator/verilator.git /tmp/verilator && \
    cd /tmp/verilator && autoconf && ./configure --prefix=/usr/local && \
    make -j"$(nproc)" && make install && rm -rf /tmp/verilator
# Upstream installs the lockfile with `uv pip sync`. Same lockfile installed with
# pip instead: uv's static binary segfaults under the QEMU emulation BuildKit uses
# for amd64 builds on arm64 hosts. pip enforces every hash, but cannot hash the
# lockfile's one VCS pin (cocotbext-pcie @ git+…@<commit>), so that line is
# installed separately at the same pinned commit, with --no-deps (its deps are
# already in the hashed set).
RUN curl -fsSL "https://raw.githubusercontent.com/NVlabs/cvdp_benchmark/${CVDP_REPO_REV}/docker/requirements.txt" \
      -o /tmp/requirements.txt && \
    grep -E '^[a-zA-Z0-9_.-]+ @ git\+' /tmp/requirements.txt | sed 's/^[^@]*@ //' > /tmp/vcs.txt && \
    test "$(wc -l < /tmp/vcs.txt)" = 1 && \
    sed -E '/^[a-zA-Z0-9_.-]+ @ git\+/d' /tmp/requirements.txt > /tmp/hashed.txt && \
    python3 -m venv /venv && \
    /venv/bin/pip install --no-cache-dir --require-hashes -r /tmp/hashed.txt && \
    /venv/bin/pip install --no-cache-dir --no-deps -r /tmp/vcs.txt && \
    rm /tmp/requirements.txt /tmp/vcs.txt /tmp/hashed.txt
# The simulator toolchain on PATH for every user (the agent is told it can
# compile and simulate); /venv carries cocotb + pytest.
ENV PATH=/venv/bin:/usr/local/bin:$PATH

# Shared grader — root-only (benchmarks rule 5): runs a task's own CVDP harness.
# It runs on /usr/bin/python3 with Ubuntu's PyYAML, leaving upstream's /venv as
# locked.
RUN apt-get update && apt-get install -y --no-install-recommends python3-yaml \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /tests /code
COPY cvdp_grade.py /tests/cvdp_grade.py
RUN chmod 700 /tests && chmod 600 /tests/cvdp_grade.py && chown -R root:root /tests
WORKDIR /code
