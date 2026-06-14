# syntax=docker/dockerfile:1
# SkillsBench (benchflow-ai/skillsbench)
# 86 tasks across 11 domains. Per-task images: EVAL_TASK_ID is a build ARG
# set to the task name (e.g. "citation-check"). Each image bakes exactly one
# task's input files, instruction, and test suite.
#
# Build the shared base first (once), then per-task images are fast (~10s):
#   docker build --target skills-bench-base \
#     -t local/benchmarks/skills-bench-base:latest benchmarks/skills-bench/
#   docker build --build-arg EVAL_TASK_ID=citation-check \
#     -t local/benchmarks/skills-bench-citation-check:latest benchmarks/skills-bench/
#
# SKILLS_BENCH_REVISION pins the upstream repo commit for reproducibility.
# Get the current SHA: git ls-remote https://github.com/benchflow-ai/skillsbench HEAD

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint

# ── Shared base: all heavy deps, cloned repo ─────────────────────────────────
# Built once. Per-task images FROM this stage so pyscipopt, node, jdk etc.
# are in the cache and don't recompile on every task build.
FROM ubuntu:24.04 AS skills-bench-base

ARG SKILLS_BENCH_REVISION=312d07e15e5398f6eda32ee1bb86e492ab18edd1

ENV DEBIAN_FRONTEND=noninteractive

# System deps: union across all supported tasks
# libgfortran5: pyscipopt (bike-rebalance)
# nodejs/npm: data-to-d3, fix-visual-stability, react-performance-debugging,
#             threejs-*, suricata-custom-exfil, radar-vital-signs, fix-build-agentops
# default-jdk: fix-build-google-auto, fix-druid-loophole-cve, flink-query,
#              python-scala-translation, spring-boot-jakarta-migration
# graphviz: dialogue-parser
# ffmpeg: video-silence-remover, multilingual-video-dubbing
# poppler-utils: edit-pdf, pdf-excel-diff, latex-formula-extraction
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           git curl ca-certificates wget \
           python3 python3-pip python3-venv \
           libgfortran5 \
           nodejs npm \
           default-jdk \
           graphviz \
           ffmpeg \
           poppler-utils \
           libgomp1; \
      then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || exit 1; \
    rm -rf /var/lib/apt/lists/*

# uv + pytest venv with all verifier deps pre-installed
RUN pip3 install --no-cache-dir --break-system-packages uv==0.7.13
RUN uv venv /opt/pytest-venv && \
    uv pip install --no-cache --python /opt/pytest-venv/bin/python \
      pytest==8.4.1 \
      pytest-json-ctrf==0.3.5 \
      pyscipopt==6.1.0

# Common task deps (union across all tasks that don't install their own)
RUN pip3 install --no-cache-dir --break-system-packages \
      requests==2.32.3 \
      bibtexparser==1.4.2 \
      pyscipopt==6.1.0 \
      pandas \
      numpy \
      scipy \
      matplotlib \
      openpyxl \
      python-pptx \
      Pillow \
      PyPDF2 \
      pypdf \
      pymupdf \
      reportlab \
      sympy \
      networkx \
      scikit-learn \
      pyarrow

# Clone the benchmark repo at the pinned revision into the base layer
RUN git clone --depth=1 https://github.com/benchflow-ai/skillsbench.git /opt/skillsbench \
    && (git -C /opt/skillsbench checkout "${SKILLS_BENCH_REVISION}" 2>/dev/null || true)

COPY --from=entrypoint /eval-materialize-task /usr/local/bin/eval-materialize-task
RUN chmod +x /usr/local/bin/eval-materialize-task

# ── Per-task image ────────────────────────────────────────────────────────────
FROM skills-bench-base:latest AS final

ARG EVAL_TASK_ID

ENV EVAL_BENCHMARK=skills-bench
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="skills-bench"
LABEL eval.benchmark.description="SkillsBench - expert tasks across software engineering, math, cybersecurity, and more"
LABEL eval.benchmark.tasks="86"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.data_revision="312d07e15e5398f6eda32ee1bb86e492ab18edd1"
LABEL eval.benchmark.url="https://www.skillsbench.ai/"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2602.12670"

# Install task-specific extra deps from the upstream Dockerfile (if any).
# We source the task's own Dockerfile for RUN pip/apt lines so each task
# gets exactly the deps its tests need without us having to enumerate them.
# Tasks that need torch or erlang are excluded (too heavy / custom builds).
RUN df=/opt/skillsbench/tasks/${EVAL_TASK_ID}/environment/Dockerfile; \
    if [ -f "$df" ]; then \
      grep -E "^RUN pip" "$df" | grep -v "torch\|trl\|pyopenjtalk\|elan\|lean" \
        | sed 's/^RUN //' | while IFS= read -r cmd; do \
            eval "$cmd" 2>/dev/null || true; \
          done; \
    fi

# Task input files → /root (agent-visible working dir)
# Flat files from environment/ → /root
# data/ tree → /data/ (civ6 and similar tasks)
RUN mkdir -p /root /tasks /tests /data \
    && find /opt/skillsbench/tasks/${EVAL_TASK_ID}/environment \
            -maxdepth 1 -type f ! -name Dockerfile \
            -exec cp {} /root/ \; \
    && if [ -d /opt/skillsbench/tasks/${EVAL_TASK_ID}/environment/data ]; then \
         cp -r /opt/skillsbench/tasks/${EVAL_TASK_ID}/environment/data/. /data/; \
       fi \
    && chmod -R a+rX /root /data

# Test suite → /tests (root-owned, agent cannot read)
RUN cp /opt/skillsbench/tasks/${EVAL_TASK_ID}/tests/test_outputs.py /tests/test_outputs.py \
    && find /opt/skillsbench/tasks/${EVAL_TASK_ID}/tests \
            -maxdepth 1 -type f ! -name test.sh ! -name test_outputs.py \
            -exec cp {} /tests/ \; 2>/dev/null || true \
    && for d in src ground_truths; do \
         if [ -d /opt/skillsbench/tasks/${EVAL_TASK_ID}/tests/$d ]; then \
           cp -r /opt/skillsbench/tasks/${EVAL_TASK_ID}/tests/$d /tests/$d; \
         fi; \
       done \
    && chmod -R 600 /tests

# Task instruction → /tasks (root-only)
RUN cp /opt/skillsbench/tasks/${EVAL_TASK_ID}/instruction.md /tasks/instruction.txt \
    && echo "${EVAL_TASK_ID}" > /tasks/id.txt \
    && chmod -R 600 /tasks

# Grade script
RUN cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier/scores
export PYTHONPATH="/tests/src:${PYTHONPATH:-}"
cd /root
/opt/pytest-venv/bin/pytest \
  --ctrf /logs/verifier/ctrf.json \
  /tests/test_outputs.py -rA -v \
  && PYTEST_OK=1 || PYTEST_OK=0
# civ6-adjacency-optimizer writes a continuous score to scores/scenario_3.txt
if [ -f /logs/verifier/scores/scenario_3.txt ]; then
  cat /logs/verifier/scores/scenario_3.txt > /logs/verifier/reward.txt
else
  echo $PYTEST_OK > /logs/verifier/reward.txt
fi
GRADE
RUN chmod +x /grade.sh

# Entrypoint: exports TASK from baked instruction
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/instruction.txt)"
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh

RUN chmod 777 /root

WORKDIR /root
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
