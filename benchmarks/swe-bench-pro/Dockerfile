# syntax=docker/dockerfile:1
# SWE-bench Pro
# Harder, contamination-resistant subset of SWE-bench with commercial repos.
# Per-task images (task ID in image name).
#
# TODO: Upstream per-task image registry path is tentative. The official
# SWE-bench Pro release ships instance-level Docker images; update the
# base-image namespace once the public registry is finalized.

ARG TASK_ID
FROM ghcr.io/swe-bench/swe-bench-pro.eval.x86_64.${TASK_ID}:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="swe-bench-pro"
LABEL dock.benchmark.description="SWE-bench Pro - harder, professional-grade software engineering tasks"
LABEL dock.benchmark.tasks="731"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/SWE-bench/SWE-bench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2509.16941"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/swe_bench"

ENV BENCHMARK=swe-bench-pro

# Install grading deps to root-only path (agent cannot access or tamper)
RUN pip install --no-cache-dir --target /tests/deps swebench 2>/dev/null || \
    pip3 install --no-cache-dir --target /tests/deps swebench

RUN apt-get update && apt-get install -y curl jq 2>/dev/null || true
ARG TASK_ID
ENV TASK_ID=${TASK_ID}

# Fetch this task's metadata from the HuggingFace rows API.
# TODO: confirm final dataset repo name once SWE-bench Pro is published.
RUN mkdir -p /tasks/0 && \
    for offset in 0 100 200 300 400 500 600 700; do \
      curl -sf "https://datasets-server.huggingface.co/rows?dataset=SWE-bench/SWE-bench_Pro&config=default&split=test&offset=$offset&length=100" > /tmp/page.json && \
      if jq -e --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id)' /tmp/page.json > /dev/null 2>&1; then \
        jq -r --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.instance_id' /tmp/page.json > /tasks/0/id.txt && \
        jq -r --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row.problem_statement' /tmp/page.json > /tasks/0/problem.txt && \
        jq --arg id "${TASK_ID}" '.rows[] | select(.row.instance_id == $id) | .row' /tmp/page.json > /tasks/0/config.json && \
        break; \
      fi; \
    done && rm -f /tmp/page.json || true
RUN chmod -R 600 /tasks

# Test: apply test_patch, run tests, grade using swebench harness (same shape as SWE-bench)
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
subprocess.run('git diff > /output/agent/patch.diff', shell=True)
os.environ['PATH'] = '/opt/miniconda3/envs/testbed/bin:' + os.environ.get('PATH', '')

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

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
chown -R agent:agent /testbed 2>/dev/null || true

if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files.

$(cat /tasks/0/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
