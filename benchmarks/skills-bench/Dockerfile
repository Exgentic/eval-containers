# syntax=docker/dockerfile:1
# SkillsBench (benchflow-ai/skillsbench)
# 94 tasks across 11 domains. Per-task images: EVAL_TASK_ID is a build ARG
# set to the task name (e.g. "citation-check"). Each image bakes exactly one
# task's input files, instruction, and test suite.
#
# Currently supported tasks: citation-check
#
# Build:
#   docker build --build-arg EVAL_TASK_ID=citation-check \
#     -t local/benchmark-skills-bench:citation-check benchmarks/skills-bench/
#
# SKILLS_BENCH_REVISION pins the upstream repo commit for reproducibility.
# Get the current SHA: git ls-remote https://github.com/benchflow-ai/skillsbench HEAD

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint

FROM ubuntu:24.04
ARG EVAL_TASK_ID
ARG SKILLS_BENCH_REVISION=312d07e15e5398f6eda32ee1bb86e492ab18edd1

ENV DEBIAN_FRONTEND=noninteractive
ENV EVAL_BENCHMARK=skills-bench
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="skills-bench"
LABEL eval.benchmark.description="SkillsBench - expert tasks across software engineering, math, cybersecurity, and more"
LABEL eval.benchmark.tasks="94"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.data_revision="312d07e15e5398f6eda32ee1bb86e492ab18edd1"
LABEL eval.benchmark.url="https://www.skillsbench.ai/"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2602.12670"

COPY --from=entrypoint /eval-materialize-task /usr/local/bin/eval-materialize-task
RUN chmod +x /usr/local/bin/eval-materialize-task

# ── System deps ──────────────────────────────────────────────────────────────
# Union of deps across currently-supported tasks + uv pre-installed so
# grade.sh never needs to curl it at runtime.
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing && apt-get install -y --no-install-recommends \
           git curl ca-certificates wget \
           python3 python3-pip python3-venv; \
      then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || exit 1; \
    rm -rf /var/lib/apt/lists/*

# Pre-install uv and the pytest deps into a venv so the verifier runs fully offline.
RUN pip3 install --no-cache-dir --break-system-packages uv==0.7.13
RUN uv venv /opt/pytest-venv && \
    uv pip install --no-cache --python /opt/pytest-venv/bin/python \
      pytest==8.4.1 \
      pytest-json-ctrf==0.3.5

# ── citation-check deps (requests, bibtexparser) ─────────────────────────────
# Mirrors tasks/citation-check/environment/Dockerfile exactly.
RUN pip3 install --no-cache-dir --break-system-packages \
      requests==2.32.3 \
      bibtexparser==1.4.2

# ── Fetch the benchmark repo at the pinned revision ──────────────────────────
RUN git clone --depth=1 https://github.com/benchflow-ai/skillsbench.git /tmp/skillsbench \
    && (git -C /tmp/skillsbench checkout "${SKILLS_BENCH_REVISION}" 2>/dev/null || true)

# ── Task input files → /root (agent-visible working dir) ─────────────────────
# Copies every non-Dockerfile file from environment/ into /root, which is
# where the upstream task expects its inputs (e.g. test.bib → /root/test.bib).
RUN mkdir -p /root /tasks /tests \
    && find /tmp/skillsbench/tasks/${EVAL_TASK_ID}/environment \
            -maxdepth 1 -type f ! -name Dockerfile \
            -exec cp {} /root/ \; \
    && chmod -R a+rX /root

# ── Test suite → /tests (root-owned, agent cannot read) ──────────────────────
RUN cp /tmp/skillsbench/tasks/${EVAL_TASK_ID}/tests/test_outputs.py /tests/test_outputs.py \
    && find /tmp/skillsbench/tasks/${EVAL_TASK_ID}/tests \
            -maxdepth 1 -type f ! -name test.sh ! -name test_outputs.py \
            -exec cp {} /tests/ \; 2>/dev/null || true \
    && chmod -R 600 /tests

# ── Task instruction → /tasks (root-only) ────────────────────────────────────
# The entrypoint reads /tasks/instruction.txt and exports it as $TASK so the
# agent receives it via process-compose's sanitized env (never via filesystem).
RUN cp /tmp/skillsbench/tasks/${EVAL_TASK_ID}/instruction.md /tasks/instruction.txt \
    && echo "${EVAL_TASK_ID}" > /tasks/id.txt \
    && chmod -R 600 /tasks

RUN rm -rf /tmp/skillsbench

# ── Grade script ──────────────────────────────────────────────────────────────
# Replaces the upstream test.sh. Uses the pre-installed uv rather than
# downloading it, so the verifier phase is fully offline.
RUN cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
cd /root
/opt/pytest-venv/bin/pytest \
  --ctrf /logs/verifier/ctrf.json \
  /tests/test_outputs.py -rA -v \
  && echo 1 > /logs/verifier/reward.txt \
  || echo 0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh

# ── Entrypoint ────────────────────────────────────────────────────────────────
# Exports TASK from the baked instruction. Input files are already in /root.
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
