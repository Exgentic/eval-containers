# syntax=docker/dockerfile:1
# AssertLLM2 harness — what every assertllm2-verify benchmark is built on.
#
# The tasks (a spec and the RTL of one design variant each), the grader and the
# entrypoint. A benchmark on top of it adds only its name and the tools the
# agent gets, so all of them grade the same tasks against the same labels.
#
# Upstream (github.com/hkust-zhiyao/AssertLLM2) is an assertion-generation
# benchmark; its designs, mutants and merged multi-bug variants are reused here
# as correct/incorrect verdicts. How the tasks are built: build-tasks.py; what
# each must pass to be one: check-tasks.py; the label assumption: README.md of
# assertllm2-verify.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Rule 4: one ARG drives both the checkout and the data_revision label.
ARG BENCHMARK_VERSION=9c481ca98fa0a78f248fcf2022127ab80f1e02df
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}sby:latest AS sby
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-github:latest AS base

# ─── Build the tasks (nothing here reaches the final image but the tasks) ───
FROM base AS tasks
ARG BENCHMARK_VERSION
# The tools the setups ship, at the same versions (the same Debian release),
# so every task is checked with what an agent could run on it. python3-click is
# SymbiYosys's one dependency; it runs on Debian's python3, which yosys pulls in.
RUN apt-get update && apt-get install -y --no-install-recommends \
      iverilog verilator yosys z3 python3-click \
 && rm -rf /var/lib/apt/lists/*
COPY --from=sby /bundle/ /
# Upstream wrote its mutants with PyVerilog 1.2.1 plus its own patch to it;
# the same emitter puts the golden in the mutants' layout (build-tasks.py).
RUN pip install --no-cache-dir pyverilog==1.2.1 jinja2==3.1.6 markupsafe==3.0.3 ply==3.11
RUN install -d -m 700 /src \
 && curl -fsSL "https://github.com/hkust-zhiyao/AssertLLM2/archive/${BENCHMARK_VERSION}.tar.gz" \
  | tar -xz --strip-components=1 -C /src \
 && P=$(python3 -c 'import os, pyverilog; print(os.path.dirname(pyverilog.__file__))') \
 && cp /src/mutation/pyverilog_change/ast.py /src/mutation/pyverilog_change/parser.py \
       /src/mutation/pyverilog_change/ast_classes.txt "$P/vparser/" \
 && cp /src/mutation/pyverilog_change/codegen.py "$P/ast_code_generator/"
COPY build-tasks.py check-tasks.py designs.txt tasks.txt /build/
WORKDIR /build
RUN mkdir /out && python3 build-tasks.py /src designs.txt /out "${BENCHMARK_VERSION}" \
 && python3 check-tasks.py /out/all.jsonl tasks.txt /out/tasks.jsonl /out/checks.jsonl

# ─── The harness ───
FROM base
ARG BENCHMARK_VERSION
LABEL eval.benchmark.url="https://github.com/hkust-zhiyao/AssertLLM2"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2605.27472"
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"

# Root only: every variant of every design, its label, and how it was built
# and checked. The agent sees one task's files, laid out in /app.
COPY --from=tasks /out/tasks.jsonl /tasks/all.jsonl
COPY --from=tasks /out/assets /tasks/assets
COPY --from=tasks /out/report.json /out/checks.jsonl /tasks/build/
RUN chown -R root:root /tasks && chmod -R go= /tasks

# build-tasks.py too: the oracle rebuilds a design's golden with it.
COPY grade.py build-tasks.py /tests/
RUN chmod -R go= /tests
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

# ASSERTLLM2_SPEC=0 leaves /app/spec.md and the figures out (assertllm2-verify-nospec).
ENV ASSERTLLM2_SPEC=1
ENV EVAL_TASK_PROMPT="You are verifying a hardware design against its specification. /app/spec.md is the specification (figures it refers to are under /app/figures/). /app/rtl/ holds the design's complete Verilog RTL: the top module is {top}, and /app/rtl/files.f lists its source files in compile order, with +incdir+include, as Icarus Verilog, Verilator and Yosys accept it. Decide whether this RTL implements the specification correctly. {tools} Judge the RTL as supplied; do not fix it. When you are done, write /home/agent/answer.json containing one JSON object with two fields: \"verdict\", either \"correct\" or \"incorrect\", and \"explanation\", a brief account for a person reviewing your verdict: why the relevant behavior conforms, or which requirement is violated and how. Only the verdict is scored."
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
