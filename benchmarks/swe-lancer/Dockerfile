# syntax=docker/dockerfile:1
# SWE-Lancer (OpenAI)
# Real-world freelance software-engineering tasks (Expensify codebase, Playwright
# E2E tests). Upstream source: github.com/openai/preparedness (project/swelancer).
# The legacy repo github.com/openai/SWELancer-Benchmark now redirects there.
#
# Per-task build from source (benchmarks/RULES.md 24g): no per-task upstream
# images exist, so build.sh builds the shared base ONCE (swelancer_x86 — conda /
# Python / Playwright / Node / Ruby + the bundled issues/ + harness, from
# openai/preparedness at a pinned commit) and this Dockerfile overlays the
# per-task setup + eval pipeline on it:
#   1. ansible-playbook setup_expensify.yml — check out Expensify at the task's
#      commit, re-introduce the bug, npm install + webpack (upstream's own steps).
#   2. bake the task prompt + grader; lock the tests/issue data root-only.
# The gold solution is never baked (solution.sh fetches it at oracle time).
ARG TASK_BASE=swelancer_x86:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${TASK_BASE}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-lancer"
LABEL eval.benchmark.upstream_base="swelancer_x86 (built from openai/preparedness@project/swelancer, pinned)"
LABEL eval.benchmark.description="SWE-Lancer - real-world freelance SWE tasks (OpenAI)"
LABEL eval.benchmark.tasks="198"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/openai/preparedness"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.12115"
ENV EVAL_BENCHMARK=swe-lancer

# The task identity (real issue id) is baked as ISSUE_ID for the grader + gold,
# but NOT exposed to the agent (the framework launches the agent with `env -i`,
# rule 7). EVAL_TASK_ID is overridden to 0 at oracle runtime, so the grader/gold
# read the baked ISSUE_ID, never EVAL_TASK_ID. PREP_REF pins the upstream commit
# so solution.sh fetches the matching gold patch.
ARG EVAL_TASK_ID
ARG PREP_REF
ENV EVAL_TASK_ID=${EVAL_TASK_ID}
ENV ISSUE_ID=${EVAL_TASK_ID}
ENV PREP_REF=${PREP_REF}

# Per-task setup (upstream Dockerfile_x86_per_task steps): check out Expensify at
# issues/$ISSUE_ID/commit_id.txt, re-introduce the bug, npm install + webpack.
RUN ansible-playbook -i "localhost," --connection=local /app/tests/setup_expensify.yml

# Task prompt for the entrypoint (description from the bundled issue_data.json).
RUN mkdir -p /tasks/0 \
 && python3 -c "import json,os,pathlib; p=pathlib.Path('/app/tests/issues/'+os.environ['ISSUE_ID']+'/issue_data.json'); d=json.loads(p.read_text()) if p.exists() else {}; pathlib.Path('/tasks/0/problem.txt').write_text(d.get('description') or d.get('title') or os.environ['ISSUE_ID']); pathlib.Path('/tasks/0/id.txt').write_text(os.environ['ISSUE_ID'])" \
 && chmod -R 600 /tasks

# Lock the upstream tests + issue data root-only so the agent can't read its
# target test.py / bug patch and game the grade (isolation, rule 7). The grader
# and the (mounted, fetched) gold solution run as root / fetch fresh.
RUN chmod -R 700 /app/tests && chown -R root:root /app/tests

COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
