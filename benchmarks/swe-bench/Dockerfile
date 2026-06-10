# syntax=docker/dockerfile:1
# SWE-bench Verified
# Per-task base images come from Epoch AI's PUBLIC registry on GHCR:
#   ghcr.io/epoch-research/swe-bench.eval.<arch>.<instance_id>:latest
# Layer-optimized rebuilds of the official swebench eval images — ~6x smaller,
# all 500 Verified instances, tested on x86_64; anonymously pullable, no auth.
# See https://epoch.ai/blog/swebench-docker.
#
# EVAL_TASK_ID is the instance id in its native "__" form (e.g.
# "sympy__sympy-24066") — exactly as in tasks.txt and the HuggingFace dataset,
# with no Docker-Hub "_1776_" sanitization. EVAL_BASE_ARCH selects the
# architecture: x86_64 (default — OpenShift/Linux, tested) or arm64 (Apple
# Silicon, native / no-Rosetta; best-effort & untested by Epoch — gold-check
# before trusting its grades).

ARG EVAL_TASK_ID
ARG EVAL_BASE_ARCH=x86_64
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ghcr.io/epoch-research/swe-bench.eval.${EVAL_BASE_ARCH}.${EVAL_TASK_ID}:latest

# Re-declare in-stage so the LABELs below can expand them.
ARG EVAL_TASK_ID
ARG EVAL_BASE_ARCH=x86_64

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench"
LABEL eval.benchmark.upstream_base="ghcr.io/epoch-research/swe-bench.eval.${EVAL_BASE_ARCH}.${EVAL_TASK_ID}:latest"
LABEL eval.benchmark.description="SWE-bench Verified - software engineering"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c104f840cc67f8c6eec6f759ebc8b2693d585d4a"
LABEL eval.benchmark.url="https://github.com/princeton-nlp/SWE-bench"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/swebench-verified"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/swe_bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2310.06770"

ENV EVAL_BENCHMARK=swe-bench

# System deps first (stable layer — cache-friendly)
RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*

# Install grading deps to root-only path (agent cannot access or tamper)
RUN (command -v pip3 >/dev/null && pip3 install --no-cache-dir --target /tests/deps swebench==3.0.17) \
 || pip install --no-cache-dir --target /tests/deps swebench==3.0.17

# Fetch this task's metadata from HuggingFace. EVAL_TASK_ID is already the
# dataset's native instance_id (with "__"), so it is used directly.
ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}
RUN HF_ID="${EVAL_TASK_ID}" && \
    mkdir -p /tasks/0 && \
    for offset in 0 100 200 300 400 500; do \
      curl -sf "https://datasets-server.huggingface.co/rows?dataset=princeton-nlp/SWE-bench_Verified&config=default&split=test&offset=$offset&length=100" > /tmp/page.json && \
      if jq -e --arg id "$HF_ID" '.rows[] | select(.row.instance_id == $id)' /tmp/page.json > /dev/null 2>&1; then \
        jq -r --arg id "$HF_ID" '.rows[] | select(.row.instance_id == $id) | .row.instance_id' /tmp/page.json > /tasks/0/id.txt && \
        jq -r --arg id "$HF_ID" '.rows[] | select(.row.instance_id == $id) | .row.problem_statement' /tmp/page.json > /tasks/0/problem.txt && \
        jq --arg id "$HF_ID" '.rows[] | select(.row.instance_id == $id) | .row' /tmp/page.json > /tasks/0/config.json && \
        break; \
      fi; \
    done && rm -f /tmp/page.json
RUN chmod -R 600 /tasks

# Grade with the official SWE-bench harness in two linear steps — RUN (bash)
# then GRADE (python) — with no bespoke eval/grading logic. We only ever *use*
# swebench: its per-instance eval script (bash) and its report function (python).
# The sole deviation from run_evaluation is that we run its script in THIS
# container, which already IS the testbed (the agent edited /testbed in place)
# and has no Docker daemon to launch a fresh one.
RUN cat > /grade.sh <<'SH' && chmod +x /grade.sh
#!/bin/bash
# Framework verifier contract: write the reward (1=resolved, 0=not) to
# /logs/verifier/reward.txt.
set -uo pipefail
mkdir -p /logs/verifier /output/agent
cd /testbed
# The verifier runs as root on the agent-owned repo; allow git operations here.
git config --global --add safe.directory /testbed
git diff > /output/agent/patch.diff 2>/dev/null || true
# RUN swebench's eval script (baked at build).
#  - env -u PYTHONPATH: keep the grading deps under /tests/deps from shadowing
#    the testbed's own packages.
#  - PIP_NO_BUILD_ISOLATION=1: the eval script's pip install must use the
#    setuptools/wheel already in the testbed env, not fetch them (no internet).
#  - PIP_NO_INDEX=1: never reach PyPI; every dep is baked into the image, so the
#    reinstall stays fully offline and reproducible (rules 1 & 3).
env -u PYTHONPATH PIP_NO_BUILD_ISOLATION=1 PIP_NO_INDEX=1 bash /tests/eval.sh > /logs/verifier/test_output.log 2>&1
# GRADE the log with swebench (its parser + report are a library, not a CLI).
PYTHONPATH=/tests/deps python3 /tests/grade.py /logs/verifier/test_output.log > /logs/verifier/reward.txt 2>/dev/null || echo 0 > /logs/verifier/reward.txt
SH

# Bake swebench's per-instance eval script (each image is one fixed instance):
# the exact bash run_evaluation runs inside its container — activate conda,
# reinstall the patched package, reset the test files, apply the test_patch, run
# the test command between Start/End Test markers. sys.path.insert puts
# /tests/deps before '' (cwd /testbed) so swebench's deps (e.g. requests) aren't
# shadowed by the testbed package of the same name.
RUN PYTHONPATH=/tests/deps python3 -c "import sys; sys.path.insert(0, '/tests/deps'); import json; from swebench.harness.test_spec.test_spec import make_test_spec; open('/tests/eval.sh', 'w').write(make_test_spec(json.load(open('/tasks/0/config.json'))).eval_script)"

RUN cat > /tests/grade.py <<'GRADE'
# Pure grader: score one test log with swebench's own report function (the same
# call run_evaluation makes) and print the reward — 1 iff the instance resolves.
# sys.path.insert puts /tests/deps before '' (cwd /testbed) so swebench's deps
# aren't shadowed by the testbed package of the same name.
import sys; sys.path.insert(0, '/tests/deps')
import json, os
from swebench.harness.test_spec.test_spec import make_test_spec
from swebench.harness.grading import get_eval_report
from swebench.harness.constants import KEY_INSTANCE_ID, KEY_PREDICTION

test_spec = make_test_spec(json.load(open('/tasks/0/config.json')))
patch = open('/output/agent/patch.diff').read() if os.path.exists('/output/agent/patch.diff') else ''
report = get_eval_report(test_spec, {KEY_INSTANCE_ID: test_spec.instance_id, KEY_PREDICTION: patch}, sys.argv[1], False)
print(1 if report[test_spec.instance_id]['resolved'] else 0)
GRADE

# Lock test infrastructure from agent (root-only read + execute)
RUN chmod -R 700 /tests && chown -R root:root /tests

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# Make repo writable by agent
chown -R agent:agent /testbed 2>/dev/null || true

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
