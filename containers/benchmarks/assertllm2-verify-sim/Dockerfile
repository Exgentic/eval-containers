# AssertLLM2-verify, with simulators — assertllm2-verify's tasks, plus Icarus
# Verilog and Verilator for the agent's own testbenches.
#
# Debian trixie's packages, the versions core/assertllm2-harness built and
# elaborated every task with (check-tasks.py).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}assertllm2-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assertllm2-verify-sim"
LABEL eval.benchmark.description="AssertLLM2-verify - judge RTL against its spec, with Icarus Verilog and Verilator"
LABEL eval.benchmark.tasks="19"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ENV EVAL_BENCHMARK=assertllm2-verify-sim

# make and g++ are what Verilator's generated C++ is built with.
RUN apt-get update && apt-get install -y --no-install-recommends iverilog verilator make g++ \
 && rm -rf /var/lib/apt/lists/*
ENV EVAL_TASK_TOOLS="Icarus Verilog (iverilog, vvp) and Verilator are installed: you can write testbenches and simulate the design."
