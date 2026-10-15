# syntax=docker/dockerfile:1
# KernelBench (Stanford Scaling Intelligence Lab)
# 250 tasks: replace a PyTorch model's operators with custom CUDA kernels that
# are correct and faster. Levels 1-3 of github.com/ScalingIntelligence/KernelBench
# (level 4 downloads Hugging Face models at run time). Graded by upstream's own
# evaluator, which needs an NVIDIA GPU. Rationale and GPU-free checks: README.md.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM nvidia/cuda:12.8.1-devel-ubuntu22.04

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="kernelbench"
LABEL eval.benchmark.description="KernelBench - write CUDA kernels that beat PyTorch (needs an NVIDIA GPU)"
LABEL eval.benchmark.tasks="250"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.requires="NVIDIA GPU"
LABEL eval.benchmark.url="https://github.com/ScalingIntelligence/KernelBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.10517"
LABEL eval.benchmark.harbor_impl="https://github.com/harbor-framework/harbor/pull/999"
LABEL eval.benchmark.upstream_base="docker.io/nvidia/cuda:12.8.1-devel-ubuntu22.04"
# CUDA toolkit + torch is ~12 GB unpacked; the 4Gi default evicts the build pod.
LABEL eval.build.storage="40Gi"

ENV DEBIAN_FRONTEND=noninteractive
# python3.10 is what upstream requires; python3-dev and ninja are what
# torch.utils.cpp_extension.load_inline needs to build the agent's kernels.
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-dev python3-pip ninja-build ca-certificates curl jq procps \
  && rm -rf /var/lib/apt/lists/*

# The agent builds and tests against this torch, and the grader times with it.
# cu128 matches the image's toolkit, which is the nvcc load_inline calls.
RUN pip3 install --no-cache-dir --index-url https://download.pytorch.org/whl/cu128 torch==2.9.1 \
 && pip3 install --no-cache-dir numpy==2.2.6

# The evaluator, root and grader only. Copied, not pip-installed: jammy's pip 22
# builds it as `UNKNOWN-0.0.0` with no module in it. Its LLM-client imports
# (openai, litellm) serve upstream's generation scripts and are never called
# while grading, so they get stand-ins instead of litellm's hundred dependencies.
ARG BENCHMARK_VERSION=423217d9fda91e0c2d67e4a43bf62f96f6d104f1
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
RUN curl -sfL "https://github.com/ScalingIntelligence/KernelBench/archive/${BENCHMARK_VERSION}.tar.gz" \
      | tar xz -C /tmp \
 && mv "/tmp/KernelBench-${BENCHMARK_VERSION}" /tmp/kb \
 && mkdir -p /tests/deps && cp -r /tmp/kb/src/kernelbench /tests/deps/ \
 && pip3 install --no-cache-dir --target /tests/deps \
      pydantic==2.11.7 requests==2.32.4 tqdm==4.67.1 python-dotenv==1.1.1 tomli==2.2.1 \
 && printf 'OpenAI = None\n' > /tests/deps/openai.py \
 && printf 'completion = None\n' > /tests/deps/litellm.py

# One line per task, levels in order and problems by number: task N is line N.
# The prompt is upstream's one-shot CUDA prompt, rendered by upstream's own
# constructor from the source tree (its example files resolve from there).
COPY build-tasks.py /tmp/build-tasks.py
RUN mkdir -p /tasks \
 && PYTHONPATH=/tmp/kb/src:/tests/deps python3 /tmp/build-tasks.py /tmp/kb /tasks/all.jsonl \
 && chmod 600 /tasks/all.jsonl && chmod 711 /tasks

# What this image can prove without a GPU: the grader's imports resolve, and
# nvcc builds and torch loads upstream's one-shot example kernel. The arch list
# stands in for the device nvcc would otherwise query; at grade time torch
# reads the real one.
RUN cd /tmp && PYTHONPATH=/tests/deps python3 -c \
      "import kernelbench.eval, kernelbench.kernel_static_checker" \
 && TORCH_CUDA_ARCH_LIST=8.0 TORCH_EXTENSIONS_DIR=/tmp/ext \
      python3 /tmp/kb/src/kernelbench/prompts/model_new_ex_add.py \
 && rm -rf /tmp/ext /tmp/kb /tmp/build-tasks.py /root/.cache

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
ENV EVAL_BENCHMARK=kernelbench
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
