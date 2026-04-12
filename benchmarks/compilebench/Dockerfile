# syntax=docker/dockerfile:1
# CompileBench (Quesma)
# 15 tasks — compile open source projects from source.
# Per-task images: each task has different source tarballs and build deps.
# Upstream task Dockerfiles in QuesmaOrg/CompileBench repo.
# This is a base — per-task Dockerfiles extend it or are built separately.

ARG TASK_ID
FROM ubuntu:22.04

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="compilebench"
LABEL dock.benchmark.description="CompileBench - compile open source projects"
LABEL dock.benchmark.tasks="15"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"

ENV DEBIAN_FRONTEND=noninteractive
ENV BENCHMARK=compilebench
ENV TASK_ID=${TASK_ID}

RUN apt-get update && apt-get install -y \
    build-essential autoconf automake libtool wget curl python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workdir

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
