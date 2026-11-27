# syntax=docker/dockerfile:1
# Video-MME-v2 (MME-Benchmarks, arXiv:2604.05015) — video understanding, model-only.
# Upstream: github.com/MME-Benchmarks/Video-MME-v2. Dataset: MME-Benchmarks/Video-MME-v2.
#
# 800 videos, 4 multiple-choice questions each (3200), scored per video as one
# group with upstream's grouped non-linear rating. Per-task: one image per
# video (task id = upstream video id, 001-800), because the videos total ~105 GB.
#
# Native harness (benchmarks/RULES.md 12a): upstream's protocol is a model
# call per question with the video given as 64 sampled frames, so there is no
# agent. The frames are sampled at build (materialize.py, upstream's indices)
# and only they ship — not the mp4 — so any image-capable model can run it.
# /harness.sh (run_video_mme_v2.py) asks the 4 questions through the gateway;
# /grade.sh (grade.py) applies upstream's answer extraction and group rating.
# Default config only (Video-MME-v2_64frame): no subtitles, no reasoning prompt.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Pinned dataset revision (rule 4): one ARG drives both the fetch and the label.
ARG BENCHMARK_VERSION=6e4bebb03202e1ddbf3d37703e560e51c5aa2d64
ARG EVAL_TASK_ID

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends ffmpeg; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install failed after 5 tries" >&2; exit 1; }
ARG BENCHMARK_VERSION
ARG EVAL_TASK_ID
COPY materialize.py /materialize.py
RUN python3 /materialize.py "${BENCHMARK_VERSION}" "${EVAL_TASK_ID:?video id, 001-800}" /tasks

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="video-mme-v2"
LABEL eval.benchmark.description="Video-MME-v2 - 4 multiple-choice questions on one video, seen as 64 sampled frames; upstream's grouped non-linear rating. Model-only."
LABEL eval.benchmark.tasks="800"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.agent="native"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
ARG BENCHMARK_VERSION
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/MME-Benchmarks/Video-MME-v2"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2604.05015"
ENV EVAL_BENCHMARK=video-mme-v2

COPY --from=data /tasks /tasks
COPY grade.py /tests/grade.py
RUN chmod -R 600 /tasks /tests && find /tasks /tests -type d -exec chmod 700 {} +

COPY run_video_mme_v2.py /app/run_video_mme_v2.py
RUN chmod 755 /app/run_video_mme_v2.py && \
    printf '#!/bin/bash\nexec python3 /app/run_video_mme_v2.py\n' > /harness.sh && \
    printf '#!/bin/bash\nexec python3 /tests/grade.py\n' > /grade.sh && \
    printf '#!/bin/bash\nexec "$@"\n' > /entrypoint.sh && \
    chmod +x /harness.sh /grade.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
