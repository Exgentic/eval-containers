# syntax=docker/dockerfile:1
# AlgoTune (Press et al., NeurIPS 2025)
# 154 tasks: make a numerical or algorithmic function faster than its reference
# while keeping its output valid. CPU only, graded by upstream's own evaluator.
# Rationale: README.md.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM python:3.12-slim

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="algotune"
LABEL eval.benchmark.description="AlgoTune - make numerical and algorithmic functions faster than their reference"
LABEL eval.benchmark.tasks="154"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.url="https://github.com/oripress/AlgoTune"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2507.15887"
LABEL eval.benchmark.harbor_impl="https://github.com/harbor-framework/harbor/tree/main/adapters/algotune"

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ca-certificates curl git jq procps \
  && rm -rf /var/lib/apt/lists/*

# Upstream's own pinned environment, minus the LLM clients its agent uses and the
# CUDA wheels; torch comes from the CPU index at the same version. Upstream pins
# pot 1.0.0, which PyPI does not have: its newest, 0.9.7.post1, is what Harbor gets.
ARG BENCHMARK_VERSION=dff9914c10800c7a031c9e8c3d4d1c8cd1b38906
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
RUN curl -sfL "https://github.com/oripress/AlgoTune/archive/${BENCHMARK_VERSION}.tar.gz" | tar xz -C /tmp \
 && mv "/tmp/AlgoTune-${BENCHMARK_VERSION}" /tmp/at \
 && grep -vE '^(nvidia-|triton|torch==|litellm|openai|google-)' /tmp/at/requirements.txt \
      | sed 's/^pot==1.0.0$/pot==0.9.7.post1/' > /tmp/requirements.txt \
 && pip install --no-cache-dir --no-deps --index-url https://download.pytorch.org/whl/cpu \
      "$(grep -E '^torch==' /tmp/at/requirements.txt)" \
 && pip install --no-cache-dir --no-deps -r /tmp/requirements.txt

# Task N is line N: the tasks sorted by name. Upstream's task classes and its
# evaluator go to /tests, root and grader only.
COPY build-tasks.py /tmp/build-tasks.py
RUN mkdir -p /tasks /tests/upstream \
 && python3 /tmp/build-tasks.py /tmp/at /tasks/all.jsonl \
 && chmod 600 /tasks/all.jsonl && chmod 711 /tasks \
 && cp -r /tmp/at/AlgoTuner /tmp/at/AlgoTuneTasks /tests/upstream/ \
 && rm -rf /tmp/at /tmp/build-tasks.py /tmp/requirements.txt /root/.cache

RUN groupadd -g 1004 grader && useradd -u 1004 -g 1004 -M -s /usr/sbin/nologin grader \
 && mkdir -p /work && chmod 711 /work
COPY grade.py /tests/grade.py
RUN chown -R root:grader /tests && chmod -R u=rwX,g=rX,o= /tests

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /eval-materialize-task /grade.sh /entrypoint.sh

WORKDIR /app
ENV EVAL_BENCHMARK=algotune
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
