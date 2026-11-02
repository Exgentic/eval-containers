# syntax=docker/dockerfile:1
# BackendBench (Meta PyTorch)
# 273 tasks: implement one PyTorch aten operator as a Triton kernel that matches
# PyTorch on its OpInfo sample inputs. github.com/meta-pytorch/BackendBench's
# OpInfo correctness suite, graded by upstream's own CLI, which needs an NVIDIA
# GPU. Rationale and GPU-free checks: README.md.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM nvidia/cuda:12.8.1-devel-ubuntu22.04

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="backendbench"
LABEL eval.benchmark.description="BackendBench - write Triton kernels for PyTorch operators that pass OpInfo (needs an NVIDIA GPU)"
LABEL eval.benchmark.tasks="273"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.requires="NVIDIA GPU"
LABEL eval.benchmark.url="https://github.com/meta-pytorch/BackendBench"
LABEL eval.benchmark.paper="n/a"
LABEL eval.benchmark.upstream_base="docker.io/nvidia/cuda:12.8.1-devel-ubuntu22.04"
# CUDA toolkit + torch is ~12 GB unpacked; the 4Gi default evicts the build pod.
LABEL eval.build.storage="40Gi"

ENV DEBIAN_FRONTEND=noninteractive
# Triton JIT-compiles the agent's kernels with the C compiler and Python headers.
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-dev python3-pip build-essential ca-certificates curl jq procps \
  && rm -rf /var/lib/apt/lists/*

# The agent builds and tests against this torch (and the Triton it pins), and the
# grader runs OpInfo with it: OpInfo's samples come from the torch release itself.
# expecttest is what torch's OpInfo module imports.
RUN pip3 install --no-cache-dir --index-url https://download.pytorch.org/whl/cu128 torch==2.9.1 \
 && pip3 install --no-cache-dir numpy==2.2.6 expecttest==0.3.0

# The evaluator, root and grader only. Copied, not pip-installed: jammy's pip 22
# builds it as `UNKNOWN-0.0.0` with no module in it. Its generation-only imports
# (the LLM client's anthropic and tenacity, the TorchBench loader's datasets and
# pyarrow) are never called while grading OpInfo, so they get stand-ins.
ARG BENCHMARK_VERSION=2a8d7e19e2c13c789ff2dceb44883ea8ea04dab4
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
RUN curl -sfL "https://github.com/meta-pytorch/BackendBench/archive/${BENCHMARK_VERSION}.tar.gz" \
      | tar xz -C /tmp \
 && mkdir -p /tests/deps/tenacity \
 && cp -r "/tmp/BackendBench-${BENCHMARK_VERSION}/BackendBench" /tests/deps/ \
 && rm -rf "/tmp/BackendBench-${BENCHMARK_VERSION}" \
 && pip3 install --no-cache-dir --target /tests/deps click==8.1.8 requests==2.32.4 tqdm==4.67.1 \
 && printf 'Anthropic = AnthropicError = None\n' > /tests/deps/anthropic.py \
 && printf 'retry = lambda **kwargs: lambda f: f\n' > /tests/deps/tenacity/__init__.py \
 && printf 'wait_random_exponential = lambda **kwargs: None\n' > /tests/deps/tenacity/wait.py \
 && printf 'load_dataset = None\n' > /tests/deps/datasets.py \
 && mkdir -p /tests/deps/pyarrow && printf '' > /tests/deps/pyarrow/__init__.py \
 && printf '' > /tests/deps/pyarrow/parquet.py

# One line per task, operators sorted by name: task N is line N. The prompt is
# upstream's Triton prompt, rendered by upstream's own template manager.
COPY build-tasks.py /tmp/build-tasks.py
RUN mkdir -p /tasks \
 && cd /tmp && PYTHONPATH=/tests/deps python3 /tmp/build-tasks.py /tasks/all.jsonl \
 && chmod 600 /tasks/all.jsonl && chmod 711 /tasks

# What this image can prove without a GPU: upstream's CLI imports, and its smoke
# suite passes with PyTorch's own operators (the aten backend) on the CPU.
RUN cd /tmp && PYTHONPATH=/tests/deps python3 -m BackendBench.scripts.main \
      --suite smoke --backend aten --log-dir /tmp/smoke \
 && jq -e 'all(.[]; .is_correct or .successfully_ran)' /tmp/smoke/full_results.json \
 && rm -rf /tmp/smoke /tmp/build-tasks.py /root/.cache

# The grader executes the agent's code, so it runs as its own uid; the agent
# (1002) can read none of /tests.
RUN groupadd -g 1004 grader && useradd -u 1004 -g 1004 -M -s /usr/sbin/nologin grader \
 && mkdir -p /work && chmod 711 /work
COPY grade.py /tests/grade.py
RUN chown -R root:grader /tests && chmod -R u=rwX,g=rX,o= /tests

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /eval-materialize-task /grade.sh /entrypoint.sh

WORKDIR /app
ENV EVAL_BENCHMARK=backendbench
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
