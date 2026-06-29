# syntax=docker/dockerfile:1
# SWE-Lancer (OpenAI) — per-task PULL + overlay (rule 24g). build.sh sets
# TASK_BASE to OpenAI's prebuilt per-task image (Expensify + bug + build baked);
# this only overlays the eval pipeline.
ARG TASK_BASE=swelancer/swelancer_x86:latest
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${TASK_BASE}

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-lancer"
LABEL eval.benchmark.upstream_base="docker.io/swelancer/swelancer_x86_<task>:releasev1 (OpenAI SWE-Lancer Diamond)"
LABEL eval.benchmark.description="SWE-Lancer - real-world freelance SWE tasks (OpenAI)"
LABEL eval.benchmark.tasks="198"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/openai/preparedness"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.12115"
ENV EVAL_BENCHMARK=swe-lancer

# ISSUE_ID (baked, root-side only) drives the grader/gold; EVAL_TASK_ID is 0 at
# oracle runtime (rule 7).
ARG EVAL_TASK_ID
ENV EVAL_TASK_ID=${EVAL_TASK_ID}
ENV ISSUE_ID=${EVAL_TASK_ID}

# Task prompt for the entrypoint (from the bundled issue_data.json).
RUN mkdir -p /tasks/0 \
 && python3 -c "import json,os,pathlib; p=pathlib.Path('/app/tests/issues/'+os.environ['ISSUE_ID']+'/issue_data.json'); d=json.loads(p.read_text()) if p.exists() else {}; pathlib.Path('/tasks/0/problem.txt').write_text(d.get('description') or d.get('title') or os.environ['ISSUE_ID']); pathlib.Path('/tasks/0/id.txt').write_text(os.environ['ISSUE_ID'])" \
 && chmod -R 600 /tasks

# Lock tests + issue data root-only so the agent can't read test.py / the bug
# patch and game the grade (rule 7).
RUN chmod -R 700 /app/tests && chown -R root:root /app/tests

COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
