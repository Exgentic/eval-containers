# syntax=docker/dockerfile:1
# SWE-bench Pro
# Harder subset of SWE-bench. Upstream: github.com/scaleapi/SWE-bench_Pro-os.
#
# Upstream publishes a ready per-instance image per task on Docker Hub (the
# dataset's `dockerhub_tag` column). So this is a per-task PULL + overlay
# (swe-bench-style), not a source build: build.sh resolves the dockerhub_tag,
# passes the published image as TASK_BASE, and this Dockerfile FROMs it (repo at
# /app + git) and layers the eval pipeline. (benchmarks/RULES.md 24g.)
#
# Grading is swe-bench-pro's OWN per-instance method (NOT the standard swebench
# harness, which doesn't know these repos): a per-instance run_script.sh runs the
# repo's tests and a per-instance parser.py parses them; the task is resolved iff
# every fail_to_pass + pass_to_pass test PASSES. Both are fetched from upstream at
# a pinned commit and baked root-only — no swebench package, grading is offline.
ARG TASK_BASE
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${TASK_BASE}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench-pro"
LABEL eval.benchmark.upstream_base="docker.io/jefzda/sweap-images (scaleapi/SWE-bench_Pro-os per-instance image, tag per task)"
LABEL eval.benchmark.description="SWE-bench Pro - harder, professional-grade software engineering tasks"
LABEL eval.benchmark.tasks="1865"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/scaleapi/SWE-bench_Pro-os"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2509.16941"
ENV EVAL_BENCHMARK=swe-bench-pro

# python3 runs the grader + the per-instance parser.py (both stdlib-only); curl+jq
# fetch task metadata. No swebench package — grading is self-contained per instance.
RUN apt-get update && apt-get install -y --no-install-recommends curl jq python3 && rm -rf /var/lib/apt/lists/*

ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

# Pinned upstream refs (build.sh passes both): the HF dataset revision (task
# metadata + gold patch) and the GitHub repo commit (per-instance run_script.sh +
# parser.py).
ARG SBP_REV
ARG SBP_GH_REF

# Task metadata (problem, gold patch, test lists) from the HF dataset, pinned to
# SBP_REV. The split has 1865 rows; page until the instance is found.
RUN mkdir -p /tasks/0 && \
    for offset in 0 100 200 300 400 500 600 700 800 900 1000 1100 1200 1300 1400 1500 1600 1700 1800; do \
      curl -sf --retry 3 --retry-delay 1 "https://datasets-server.huggingface.co/rows?dataset=ScaleAI/SWE-bench_Pro&config=default&split=test&revision=${SBP_REV}&offset=$offset&length=100" > /tmp/page.json && \
      if jq -e --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id)' /tmp/page.json > /dev/null 2>&1; then \
        jq -r --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.problem_statement' /tmp/page.json > /tasks/0/problem.txt && \
        jq --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row' /tmp/page.json > /tasks/0/config.json && \
        break; \
      fi; \
    done && rm -f /tmp/page.json
RUN test -s /tasks/0/config.json && chmod -R 600 /tasks

# Per-instance test runner + parser (root-only), from upstream at SBP_GH_REF, plus
# our grader/grade.sh/entrypoint (separate files; no heredocs).
RUN mkdir -p /tests && \
    base="https://raw.githubusercontent.com/scaleapi/SWE-bench_Pro-os/${SBP_GH_REF}/run_scripts/${EVAL_TASK_ID}" && \
    curl -fsSL --retry 3 --retry-delay 1 "$base/run_script.sh" -o /tests/run_script.sh && \
    curl -fsSL --retry 3 --retry-delay 1 "$base/parser.py" -o /tests/parser.py
COPY grade.py /tests/grade.py
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh && chmod -R 700 /tests && chown -R root:root /tests

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
