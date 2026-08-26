# syntax=docker/dockerfile:1
# HWE-Bench — "SWE-bench for hardware".
# Upstream: github.com/pku-liang/hwe-bench. Dataset: henryen/hwe-bench.
#
# Real fail-to-pass RTL bug-repair cases across open-source hardware projects.
# The agent edits HDL source in a checked-out repo to fix a bug; grading is
# DETERMINISTIC by simulation (no LLM judge): a self-contained testbench fails on
# the buggy base commit and must pass once the fix is applied.
#
# Upstream publishes a ready per-PR image per case on GHCR, so this is a per-task
# PULL + overlay (swe-bench-style), not a source build (rule 24g). build.sh
# derives the base image ref + repo path from the task id (pure string transform,
# no dataset lookup) and this Dockerfile FROMs it: the RTL repo lives at
# /home/<repo>, Verilator is baked into the base image, git is present.
#
# Grading is HWE's OWN self-contained method (NOT the standard swebench harness,
# which doesn't know RTL): each case carries an inline `tb_script` that generates
# its testbench, compiles the repo's RTL with Verilator, and prints
# `TEST: <name> ... PASS|FAIL` markers. The task is resolved iff every
# fail-to-pass test PASSES. tb_script + base commit + f2p list are baked root-only
# from the pinned dataset row — grading is fully offline, no network at run time.
ARG TASK_BASE
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${TASK_BASE}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="hwe-bench"
LABEL eval.benchmark.upstream_base="ghcr.io/pku-liang/<org>_m_<repo>:pr-<number> (pku-liang/hwe-bench per-PR image, ref per task)"
LABEL eval.benchmark.description="HWE-Bench - real fail-to-pass RTL bug-repair tasks graded by simulation (SWE-bench for hardware)"
LABEL eval.benchmark.tasks="172"
LABEL eval.benchmark.env="per-task"
# Internet policy: grading is fully offline and a web fetch could leak the
# reference fix (some task ids encode their upstream source), so the agent runs
# web-tools-off. One ARG drives BOTH the declarative label and the EVAL_INTERNET
# env the agent gates on, so the two can never disagree. Because the combination
# image is `FROM` this benchmark, this ENV persists into the eval image; the
# runner allow-lists EVAL_INTERNET through to the agent (rule 7).
ARG EVAL_INTERNET=false
LABEL eval.benchmark.internet="${EVAL_INTERNET}"
ENV EVAL_INTERNET=${EVAL_INTERNET}
LABEL eval.benchmark.data_revision="82a42e0a05719366a326e09ddc668ea0d46c91f6"
LABEL eval.benchmark.url="https://github.com/pku-liang/hwe-bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2604.14709"
ENV EVAL_BENCHMARK=hwe-bench

# python3 runs the grader (stdlib-only) and extracts the dataset row; curl fetches
# the case's JSONL row at build time. No dataset package — grading is offline.
RUN apt-get update && apt-get install -y --no-install-recommends curl python3 && rm -rf /var/lib/apt/lists/*

ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

# Repo path in the base image (/home/<repo>); the grader/entrypoint need it.
ARG HWE_REPO_DIR=/home
ENV HWE_REPO_DIR=${HWE_REPO_DIR}

# Pinned henryen/hwe-bench dataset revision (task row: problem, gold fix_patch,
# base commit, tb_script, f2p tests).
ARG HWE_REV

# Upstream-cased dataset filename (<org>__<repo>), recovered by build.sh from the
# lowercase task id: EVAL_TASK_ID is Docker-safe/lowercase, but the HF JSONL files
# keep the upstream mixed case (lowRISC__ibex.jsonl, OpenXiangShan__XiangShan.jsonl).
ARG HWE_HF_SLUG

# Task metadata from the dataset, pinned to HWE_REV. The per-repo JSONL file is
# named <org>__<repo>.jsonl (HWE_HF_SLUG); select the row whose `number` matches
# the task id.
RUN mkdir -p /tasks/0 && \
    number="${EVAL_TASK_ID##*-}" && \
    curl -fsSL --retry 3 --retry-delay 1 \
      "https://huggingface.co/datasets/henryen/hwe-bench/resolve/${HWE_REV}/${HWE_HF_SLUG}.jsonl" \
      -o /tmp/cases.jsonl && \
    python3 -c "import json; n=int('${number}'); rows=[json.loads(l) for l in open('/tmp/cases.jsonl') if l.strip()]; r=next(x for x in rows if int(x['number'])==n); json.dump(r, open('/tasks/0/config.json','w'))" && \
    python3 -c "import json; open('/tasks/0/problem.txt','w').write(json.load(open('/tasks/0/config.json')).get('problem_statement',''))" && \
    rm -f /tmp/cases.jsonl
RUN test -s /tasks/0/config.json && chmod -R 600 /tasks

# Self-contained test runner (root-only) extracted from the dataset row, plus our
# grader/grade.sh/entrypoint (separate files; no heredocs).
RUN mkdir -p /tests && \
    python3 -c "import json; open('/tests/tb_script.sh','w').write(json.load(open('/tasks/0/config.json'))['tb_script'])"
COPY grade.py /tests/grade.py
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh /tests/tb_script.sh && chmod -R 700 /tests && chown -R root:root /tests

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
