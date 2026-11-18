# AssertLLM2-verify, no spec — assertllm2-verify's tasks with the specification
# and figures withheld: the agent judges the RTL from the source alone.
#
# The baseline for the spec's contribution: same tasks, labels and grader, no
# tools beyond the image's shell and Python, like assertllm2-verify.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}assertllm2-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assertllm2-verify-nospec"
LABEL eval.benchmark.description="AssertLLM2-verify - judge RTL with no spec, from the source alone"
LABEL eval.benchmark.tasks="19"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ENV EVAL_BENCHMARK=assertllm2-verify-nospec
ENV ASSERTLLM2_SPEC=0
ENV EVAL_TASK_TOOLS="No simulator or formal tool is installed: work from the source."
ENV EVAL_TASK_PROMPT="You are reviewing a hardware design. /app/rtl/ holds its complete Verilog RTL: the top module is {top}, and /app/rtl/files.f lists its source files in compile order, with +incdir+include, as Icarus Verilog, Verilator and Yosys accept it. No specification is provided. Decide whether this RTL correctly implements the design it evidently intends, or contains a bug. {tools} Judge the RTL as supplied; do not fix it. When you are done, write /home/agent/answer.json containing one JSON object with two fields: \"verdict\", either \"correct\" or \"incorrect\", and \"explanation\", a brief account for a person reviewing your verdict: why the behavior is consistent, or what the bug is and how it shows. Only the verdict is scored."
