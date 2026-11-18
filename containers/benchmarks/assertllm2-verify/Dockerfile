# AssertLLM2-verify — is this RTL a correct implementation of its spec?
#
# core/assertllm2-harness, with no tools beyond the image's shell and Python:
# the agent reads the spec and the source. assertllm2-verify-sim and
# assertllm2-verify-formal are the same tasks with a simulator or formal tools.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}assertllm2-harness:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assertllm2-verify"
LABEL eval.benchmark.description="AssertLLM2-verify - judge RTL against its spec, from the source alone"
LABEL eval.benchmark.tasks="19"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ENV EVAL_BENCHMARK=assertllm2-verify
ENV EVAL_TASK_TOOLS="No simulator or formal tool is installed: work from the specification and the source."
