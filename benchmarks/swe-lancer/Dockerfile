# syntax=docker/dockerfile:1
# SWE-Lancer (OpenAI)
# Freelance software engineering tasks (Expensify codebase, Playwright E2E
# tests). Upstream source: github.com/openai/preparedness (project/swelancer).
# The legacy repo github.com/openai/SWELancer-Benchmark now redirects there.
#
# Two-stage build flow (no private registries involved):
#   (1) Pre-build the shared base image ONCE:
#         scripts/build-swe-lancer.sh
#       clones openai/preparedness at the pinned commit, builds
#       swelancer_x86:latest from project/swelancer/Dockerfile_x86_base.
#       This installs Ubuntu+conda+Python+Playwright+Node deps and bakes in
#       the issues/ directory (per-task issue data + test files).
#   (2) Per-task build uses THIS Dockerfile, which layers on top of
#       swelancer_x86 and runs the upstream per-task playbook to check out
#       Expensify at the right commit for EVAL_TASK_ID.

ARG EVAL_TASK_ID
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM swelancer_x86:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-lancer"
LABEL eval.benchmark.upstream_base="swelancer_x86:latest (locally built from openai/preparedness@project/swelancer)"
LABEL eval.benchmark.description="SWE-Lancer - real-world freelance SWE tasks (OpenAI)"
LABEL eval.benchmark.tasks="1488"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/openai/preparedness"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.12115"
ENV EVAL_BENCHMARK_VERSION_DEFAULT=""
ENV EVAL_BENCHMARK=swe-lancer

ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}
ENV ISSUE_ID=${EVAL_TASK_ID}

# Upstream per-task setup: the base image bakes in the issues/ directory and
# setup_expensify.yml playbook. Running the playbook here checks out Expensify
# at the commit specified in issues/${ISSUE_ID}/commit_id.txt and applies the
# bug-reintroduce patch — same steps as Dockerfile_x86_per_task upstream.
RUN ansible-playbook -i "localhost," --connection=local /app/tests/setup_expensify.yml

# Record the task description (from the issue_data.json bundled in the base
# image at /app/tests/issues/${ISSUE_ID}/issue_data.json) into a eval-shaped
# task file for the entrypoint.
RUN mkdir -p /tasks/0 \
 && python3 -c "import json,os,pathlib; p=pathlib.Path('/app/tests/issues/'+os.environ['ISSUE_ID']+'/issue_data.json'); d=json.loads(p.read_text()) if p.exists() else {}; pathlib.Path('/tasks/0/problem.txt').write_text(d.get('description') or d.get('title') or os.environ['ISSUE_ID']); pathlib.Path('/tasks/0/id.txt').write_text(os.environ['ISSUE_ID'])" \
 && chmod -R 600 /tasks

# Scorer: runs the upstream Playwright E2E tests. The base image already has
# /app/tests/run.sh that executes the full test harness.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
# Capture agent's diff
(cd /app/expensify 2>/dev/null && git diff > /output/agent/patch.diff) || true

if /bin/bash -l -c '/app/tests/run.sh' > /logs/verifier/test.log 2>&1; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST

RUN chmod +x /grade.sh && chmod -R 700 /tests && chown -R root:root /tests

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
chown -R agent:agent /app/expensify 2>/dev/null || true

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are fixing a freelance software engineering task in the Expensify codebase at /app/expensify. Do NOT modify test files. When finished, your change must pass the upstream end-to-end tests.

$(cat /tasks/0/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
