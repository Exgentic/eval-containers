# syntax=docker/dockerfile:1
# SWE-bench Pro
# Harder subset of SWE-bench. Upstream: github.com/scaleapi/SWE-bench_Pro-os.
# Upstream ships per-instance Dockerfiles in TWO files that must be composed:
#   dockerfiles/base_dockerfile/instance_<id>/Dockerfile
#   dockerfiles/instance_dockerfile/instance_<id>/Dockerfile  (FROM base_<repo>)
#
# The base Dockerfile's FROM line varies per-repo (node:18, python:3.11,
# ubuntu:22.04, etc.), so we cannot write a single static FROM here that works
# for every instance. This Dockerfile instead uses a two-build flow:
#   (a) The caller/CLI first builds the concatenated upstream base+instance
#       into an image tagged "sbp/instance:${EVAL_TASK_ID}".
#   (b) THIS Dockerfile then layers the eval-specific grading machinery on top
#       of that locally-built tag — no private registries involved.
#
# For step (a), see scripts/build-swe-bench-pro.sh (clones the upstream repo
# at the pinned commit, extracts both Dockerfiles, strips the duplicate FROM,
# and runs `docker build` to produce sbp/instance:${EVAL_TASK_ID}).

ARG EVAL_TASK_ID
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM sbp/instance:${EVAL_TASK_ID}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench-pro"
LABEL eval.benchmark.upstream_base="sbp/instance:${EVAL_TASK_ID} (locally built from scaleapi/SWE-bench_Pro-os)"
LABEL eval.benchmark.description="SWE-bench Pro - harder, professional-grade software engineering tasks"
LABEL eval.benchmark.tasks="1865"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/scaleapi/SWE-bench_Pro-os"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2509.16941"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="3.0.18"
ENV EVAL_BENCHMARK=swe-bench-pro

# System deps + grading package (root-only).
RUN apt-get update && apt-get install -y --no-install-recommends curl jq && rm -rf /var/lib/apt/lists/*
RUN (command -v pip3 >/dev/null && pip3 install --no-cache-dir --target /tests/deps swebench==3.0.18) \
 || pip install --no-cache-dir --target /tests/deps swebench==3.0.18

ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}

# Task metadata from the public HF dataset.
RUN mkdir -p /tasks/0 && \
    for offset in 0 100 200 300 400 500 600 700 800 900 1000; do \
      curl -sf "https://datasets-server.huggingface.co/rows?dataset=ScaleAI/SWE-bench_Pro&config=default&split=test&offset=$offset&length=100" > /tmp/page.json && \
      if jq -e --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id)' /tmp/page.json > /dev/null 2>&1; then \
        jq -r --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.instance_id' /tmp/page.json > /tasks/0/id.txt && \
        jq -r --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.problem_statement' /tmp/page.json > /tasks/0/problem.txt && \
        jq --arg id "${EVAL_TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row' /tmp/page.json > /tasks/0/config.json && \
        break; \
      fi; \
    done && rm -f /tmp/page.json || true
RUN chmod -R 600 /tasks

RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
PYTHONPATH=/tests/deps python3 /tests/grade.py > /logs/verifier/reward.txt 2>/dev/null || echo 0 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'GRADE'
import sys; sys.path.insert(0, '/tests/deps')
import json, subprocess, shlex, re, tempfile, os
from swebench.harness.test_spec.test_spec import make_test_spec
from swebench.harness.test_spec.python import get_test_directives
from swebench.harness.constants import (MAP_REPO_VERSION_TO_SPECS, ResolvedStatus,
    EvalType, FAIL_TO_PASS, PASS_TO_PASS, KEY_INSTANCE_ID, FAIL_ONLY_REPOS,
    START_TEST_OUTPUT, END_TEST_OUTPUT)
from swebench.harness.grading import get_logs_eval, get_eval_tests_report, get_resolution_status

config = json.load(open('/tasks/0/config.json'))
spec = MAP_REPO_VERSION_TO_SPECS[config['repo']][config['version']]
test_spec = make_test_spec(config)

os.chdir('/app')
subprocess.run('git diff > /output/agent/patch.diff', shell=True)

tracked = [p for p in re.findall(r'^--- a/(.*)$', config['test_patch'], re.MULTILINE) if p != '/dev/null']
if tracked:
    subprocess.run(f"git checkout {config['base_commit']} {' '.join(shlex.quote(p) for p in tracked)}", shell=True)
subprocess.run(f"echo {shlex.quote(config['test_patch'])} | git apply -", shell=True)

test_files = get_test_directives({'repo': config['repo'], 'test_patch': config['test_patch']})
test_cmd = spec['test_cmd'] + ' ' + ' '.join(test_files)
log = tempfile.NamedTemporaryFile(mode='w', suffix='.log', delete=False)
result = subprocess.run(test_cmd, shell=True, capture_output=True, text=True)
log.write(f'{START_TEST_OUTPUT}\n{result.stdout}\n{result.stderr}\n{END_TEST_OUTPUT}')
log.close()

eval_status_map, found = get_logs_eval(test_spec, log.name)
if found:
    eval_ref = {KEY_INSTANCE_ID: test_spec.instance_id, FAIL_TO_PASS: test_spec.FAIL_TO_PASS, PASS_TO_PASS: test_spec.PASS_TO_PASS}
    eval_type = EvalType.FAIL_ONLY if test_spec.repo in FAIL_ONLY_REPOS else EvalType.PASS_AND_FAIL
    report = get_eval_tests_report(eval_status_map, eval_ref, eval_type=eval_type)
    print(1 if get_resolution_status(report) == ResolvedStatus.FULL.value else 0)
else:
    print(0)
GRADE

RUN chmod -R 700 /tests && chown -R root:root /tests

COPY --from=entrypoint /eval-entrypoint.sh /eval-entrypoint.sh
RUN chmod +x /eval-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
chown -R agent:agent /app 2>/dev/null || true

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Fix this GitHub issue in the repository at /app. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
