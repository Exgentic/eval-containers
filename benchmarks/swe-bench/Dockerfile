# syntax=docker/dockerfile:1
# SWE-bench Verified
# 500 per-task images from ghcr.io/epoch-research/swe-bench.eval.x86_64.{task_id}
# Agent fixes a bug in a real Python repo. Tests verify the fix.

ARG TASK_ID
FROM ghcr.io/epoch-research/swe-bench.eval.x86_64.${TASK_ID}:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="swebench"
LABEL dock.benchmark.description="SWE-bench Verified - software engineering"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c104f840cc67f8c6eec6f759ebc8b2693d585d4a"
LABEL dock.benchmark.url="https://github.com/princeton-nlp/SWE-bench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2310.06770"

ENV BENCHMARK=swebench

# Install grading deps to root-only path (agent cannot access or tamper)
RUN pip install --no-cache-dir --target /tests/deps swebench 2>/dev/null || \
    pip3 install --no-cache-dir --target /tests/deps swebench

# Fetch this task's metadata from HuggingFace API
RUN apt-get update && apt-get install -y curl jq 2>/dev/null || true
ARG TASK_ID
ENV TASK_ID=${TASK_ID}
RUN mkdir -p /tasks/0 && \
    for offset in 0 100 200 300 400 500; do \
      curl -sf "https://datasets-server.huggingface.co/rows?dataset=princeton-nlp/SWE-bench_Verified&config=default&split=test&offset=$offset&length=100" > /tmp/page.json && \
      if jq -e --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id)' /tmp/page.json > /dev/null 2>&1; then \
        jq -r --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.instance_id' /tmp/page.json > /tasks/0/id.txt && \
        jq -r --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.problem_statement' /tmp/page.json > /tasks/0/problem.txt && \
        jq --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row' /tmp/page.json > /tasks/0/config.json && \
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

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# Make repo writable by agent
chown -R agent:agent /testbed 2>/dev/null || true

if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
