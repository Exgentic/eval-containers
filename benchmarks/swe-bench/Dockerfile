# syntax=docker/dockerfile:1
# SWE-bench Verified
# Per-task base images come from the upstream PUBLIC Docker Hub registry at
# docker.io/swebench/sweb.eval.x86_64.<task_id>:latest — anonymously pullable,
# no auth required.
#
# NOTE: The SWE-bench harness sanitizes instance ids for Docker by replacing
# "__" with "_1776_" (see swebench.harness.test_spec.test_spec). Callers must
# pass EVAL_TASK_ID in the sanitized form used by the published images — e.g.
# "sympy_1776_sympy-24066" rather than "sympy__sympy-24066". The HuggingFace
# dataset lookup below reverses the transform so we fetch the right row.

ARG EVAL_TASK_ID
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM docker.io/swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench"
LABEL eval.benchmark.upstream_base="docker.io/swebench/sweb.eval.x86_64.${EVAL_TASK_ID}:latest"
LABEL eval.benchmark.description="SWE-bench Verified - software engineering"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c104f840cc67f8c6eec6f759ebc8b2693d585d4a"
LABEL eval.benchmark.url="https://github.com/princeton-nlp/SWE-bench"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/swebench-verified"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/swe_bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2310.06770"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="c104f840cc67f8c6eec6f759ebc8b2693d585d4a"

ENV EVAL_BENCHMARK=swe-bench

# System deps first (stable layer — cache-friendly)
RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*

# Install grading deps to root-only path (agent cannot access or tamper)
RUN (command -v pip3 >/dev/null && pip3 install --no-cache-dir --target /tests/deps swebench==3.0.17) \
 || pip install --no-cache-dir --target /tests/deps swebench==3.0.17

# Fetch this task's metadata from HuggingFace. The dataset stores instance_id
# in its human form (e.g. "sympy__sympy-24066"); EVAL_TASK_ID is the Docker-
# sanitized form (e.g. "sympy_1776_sympy-24066"). Reverse the transform for
# the dataset lookup.
ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}
RUN HF_ID="$(echo "${EVAL_TASK_ID}" | sed 's/_1776_/__/g')" && \
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

# Test: apply test_patch, run tests, grade
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
PYTHONPATH=/tests/deps python3 /tests/grade.py > /logs/verifier/reward.txt 2>/dev/null || echo 0 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

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

os.chdir('/testbed')

# Capture the agent's patch
subprocess.run('git diff > /output/agent/patch.diff', shell=True)

# Activate conda
os.environ['PATH'] = '/opt/miniconda3/envs/testbed/bin:' + os.environ.get('PATH', '')

# Reset test files to pre-PR state, apply test patch
tracked = [p for p in re.findall(r'^--- a/(.*)$', config['test_patch'], re.MULTILINE) if p != '/dev/null']
if tracked:
    subprocess.run(f"git checkout {config['base_commit']} {' '.join(shlex.quote(p) for p in tracked)}", shell=True)
subprocess.run(f"echo {shlex.quote(config['test_patch'])} | git apply -", shell=True)

# Run tests
test_files = get_test_directives({'repo': config['repo'], 'test_patch': config['test_patch']})
test_cmd = spec['test_cmd'] + ' ' + ' '.join(test_files)
log = tempfile.NamedTemporaryFile(mode='w', suffix='.log', delete=False)
result = subprocess.run(test_cmd, shell=True, capture_output=True, text=True)
log.write(f'{START_TEST_OUTPUT}\n{result.stdout}\n{result.stderr}\n{END_TEST_OUTPUT}')
log.close()

# Grade
eval_status_map, found = get_logs_eval(test_spec, log.name)
if found:
    eval_ref = {KEY_INSTANCE_ID: test_spec.instance_id, FAIL_TO_PASS: test_spec.FAIL_TO_PASS, PASS_TO_PASS: test_spec.PASS_TO_PASS}
    eval_type = EvalType.FAIL_ONLY if test_spec.repo in FAIL_ONLY_REPOS else EvalType.PASS_AND_FAIL
    report = get_eval_tests_report(eval_status_map, eval_ref, eval_type=eval_type)
    print(1 if get_resolution_status(report) == ResolvedStatus.FULL.value else 0)
else:
    print(0)
GRADE

# Lock test infrastructure from agent (root-only read + execute)
RUN chmod -R 700 /tests && chown -R root:root /tests

COPY --from=entrypoint /eval-entrypoint.sh /eval-entrypoint.sh
RUN chmod +x /eval-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# Make repo writable by agent
chown -R agent:agent /testbed 2>/dev/null || true

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
