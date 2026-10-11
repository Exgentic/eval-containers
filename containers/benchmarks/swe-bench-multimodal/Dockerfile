# syntax=docker/dockerfile:1
# SWE-bench Multimodal (test split)
# Visual, user-facing JavaScript bugs: each issue carries screenshots and the fix
# is graded by the repository's own tests. Upstream publishes one prebuilt testbed
# image per instance on Docker Hub, x86_64 only.
#
# EVAL_TASK_ID is that image's instance tag — the instance id lowercased, with
# "__" spelled "_1776_" (e.g. "openlayers_1776_openlayers-12683"; tasks.txt lists
# all of them). It is a valid image name on every surface as it stands, and the
# dataset row is found by its `image` column, so nothing maps one id to another.

ARG EVAL_TASK_ID
# Rule 4: one ARG drives both the dataset fetch and the data_revision label.
ARG BENCHMARK_VERSION=4e6662d51c48e475f7f346e4fa09a6f8b31fcaa5
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/

# Built apart from the testbed, into which nothing is installed: the grader's own
# runtime (a relocatable Python and the official harness — upstream's images do
# not all have pip to install it with), then this task's record. The runtime is
# the same for every task, so their images share its layer.
FROM --platform=linux/amd64 ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS build
RUN UV_PYTHON_INSTALL_DIR=/opt uv python install 3.12.8 \
 && mv /opt/cpython-3.12.8-linux-x86_64-gnu /tests/python \
 && uv pip install --python /tests/python/bin/python3 --target /tests/deps swebench==5.0.2
COPY grade.py /tests/
ARG EVAL_TASK_ID
ARG BENCHMARK_VERSION
COPY materialize.py /
RUN python3 /materialize.py "${BENCHMARK_VERSION}" "swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest"

FROM --platform=linux/amd64 docker.io/swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest

ARG EVAL_TASK_ID
ARG BENCHMARK_VERSION

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench-multimodal"
LABEL eval.benchmark.upstream_base="docker.io/swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest"
LABEL eval.benchmark.description="SWE-bench Multimodal - fixing visual front-end bugs from issues with screenshots"
LABEL eval.benchmark.tasks="480"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.url="https://github.com/SWE-bench/SWE-bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2410.03859"

ENV EVAL_BENCHMARK=swe-bench-multimodal
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

COPY --link --from=build /tests /tests
COPY --from=build /tasks /tasks
COPY --chmod=755 grade.sh entrypoint.sh /
# Root-only: the grader, the task record (gold patch, tests, baselines), and
# upstream's own copy of the baselines, which it ships world-readable.
# safe.directory: the verifier is root in a tree the entrypoint hands to the agent.
RUN mkdir -p /swebench && chmod 700 /tests /tasks /swebench \
 && git config --system --add safe.directory /testbed

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
