# syntax=docker/dockerfile:1
# SWE-fficiency (ICML 2026): make a workload on a real repository faster without
# breaking the repository's own tests. Per-task bases are upstream's own images,
# public on GHCR, linux/amd64 only; graded by upstream's harness. Rationale: README.md.

ARG EVAL_TASK_ID
FROM --platform=linux/amd64 ghcr.io/swefficiency/swefficiency-images:${EVAL_TASK_ID}

ARG EVAL_TASK_ID
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swefficiency"
LABEL eval.benchmark.upstream_base="ghcr.io/swefficiency/swefficiency-images:${EVAL_TASK_ID}"
LABEL eval.benchmark.description="SWE-fficiency - make a workload on a real repository faster without breaking its tests"
LABEL eval.benchmark.tasks="100"
LABEL eval.benchmark.env="per-task"
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.url="https://github.com/swefficiency/swefficiency"
LABEL eval.benchmark.paper="https://icml.cc/virtual/2026/poster/66745"
ENV EVAL_BENCHMARK=swefficiency
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

RUN apt-get update && apt-get install -y --no-install-recommends procps && rm -rf /var/lib/apt/lists/*

# Upstream's harness at its locked dependencies (uv.lock), root-only, on conda's
# base Python; the agent gets only the testbed env.
ARG HARNESS_REVISION=12d32a2d6800824a7d84bdb6797b5708e7b7957f
RUN curl -sfL "https://github.com/swefficiency/swefficiency/archive/${HARNESS_REVISION}.tar.gz" | tar xz -C /tmp \
 && /opt/miniconda3/bin/python -m pip install --no-cache-dir --target /tmp/uv uv==0.13.1 \
 && cd "/tmp/swefficiency-${HARNESS_REVISION}" \
 && /tmp/uv/bin/uv export --frozen --no-hashes --no-dev --extra inference --no-emit-project -o /tmp/requirements.txt \
 && /opt/miniconda3/bin/python -m pip install --no-cache-dir --target /tests/deps -r /tmp/requirements.txt \
 && /opt/miniconda3/bin/python -m pip install --no-cache-dir --no-deps --target /tests/deps . \
 && cd / && rm -rf /tmp/uv /tmp/requirements.txt "/tmp/swefficiency-${HARNESS_REVISION}"

# The task: the dataset row, with the expert patch the grader times, and upstream's
# agent prompt.
ARG BENCHMARK_VERSION=7fbfe6c2f1f409afb17acc4c3f737991ec6e1c27
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
COPY build-task.py /tmp/build-task.py
RUN PYTHONPATH=/tests/deps /opt/miniconda3/bin/python /tmp/build-task.py \
      "${EVAL_TASK_ID}" "${BENCHMARK_VERSION}" "${HARNESS_REVISION}" \
 && rm -rf /tmp/build-task.py /root/.cache/huggingface && chmod -R go= /tasks

# Grading is offline: build requirements and workload data are fetched now.
COPY offline.py /tmp/offline.py
RUN /opt/miniconda3/bin/python /tmp/offline.py && rm /tmp/offline.py
ENV PIP_FIND_LINKS=/opt/wheelhouse PIP_NO_INDEX=1

# Upstream's image keeps the repository's later history, which can hold the fix:
# drop every ref that is not part of the task's own history, then the objects.
RUN cd /testbed && git remote | xargs -r -n1 git remote remove \
 && git for-each-ref --format='%(refname)' | while read -r ref; do \
      git merge-base --is-ancestor "$ref" HEAD || git update-ref -d "$ref"; done \
 && git reflog expire --expire=now --all && git gc --prune=now --quiet \
 && test "$(git rev-list --all --count)" = "$(git rev-list HEAD --count)" \
 && git rev-parse HEAD > /tests/head

COPY grade.py /tests/
RUN mkdir -p /tests/pristine && chmod -R go= /tests

COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
