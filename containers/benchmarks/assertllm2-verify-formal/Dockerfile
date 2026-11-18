# AssertLLM2-verify, with formal tools — assertllm2-verify's tasks, plus Yosys,
# SymbiYosys and Z3, for properties the agent writes and model-checks.
#
# Debian trixie's yosys (with yosys-abc and yosys-smtbmc) and z3, and sby from
# core/sby, which runs on Debian's python3 with python3-click: the toolchain
# core/assertllm2-harness elaborated every task with (check-tasks.py).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}sby:latest AS sby
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}assertllm2-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assertllm2-verify-formal"
LABEL eval.benchmark.description="AssertLLM2-verify - judge RTL against its spec, with Yosys, SymbiYosys and Z3"
LABEL eval.benchmark.tasks="19"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ENV EVAL_BENCHMARK=assertllm2-verify-formal

RUN apt-get update && apt-get install -y --no-install-recommends yosys z3 python3-click \
 && rm -rf /var/lib/apt/lists/*
COPY --from=sby /bundle/ /
ENV EVAL_TASK_TOOLS="Yosys, SymbiYosys (sby) and the Z3 solver are installed: you can write SystemVerilog assertions and check them formally (sby engines smtbmc z3, or abc)."
