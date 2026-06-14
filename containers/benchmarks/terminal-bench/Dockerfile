# syntax=docker/dockerfile:1
# Terminal-Bench 2.1 (Harbor) — per-task eval overlay.
#
# Source: github.com/harbor-framework/terminal-bench-2-1. Each task ships an
# environment/Dockerfile (the task env, heterogeneous base + setup), a direct
# instruction.md, tests/ (test.sh + test_outputs.py), and solution/solve.sh.
# No per-task upstream images exist, so the per-task build is two steps
# (build.sh): (1) build environment/Dockerfile -> the task env; (2) THIS file
# overlays the eval pipeline on it (TASK_BASE = the env image).
#
# We add ONLY: the instruction (TASK), a root-only copy of the test harness, the
# grader, and the entrypoint. The gold solution is NEVER baked, and the raw tests
# are root-only (chmod 700) — the agent can read neither the answer nor the tests
# (benchmarks/RULES.md rule 9 / 24g; see core/oracle, solution.sh).
ARG TASK_BASE
FROM ${TASK_BASE}
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="terminal-bench"
LABEL eval.benchmark.tasks="89"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/harbor-framework/terminal-bench-2-1"
# EVAL_TASK_ID is consumed here (the per-task marker, benchmarks/RULES.md 24a/24f).
# TBENCH_TASK pins the real task name for solution.sh: the oracle/runner override
# EVAL_TASK_ID at run time (their /tasks index), so the gold fetcher can't rely on
# it. TBENCH_REF is the pinned dataset commit. build.sh passes both as build args.
ARG EVAL_TASK_ID
ARG TBENCH_REF
ENV EVAL_BENCHMARK=tbench \
    EVAL_TASK_ID=${EVAL_TASK_ID} \
    TBENCH_TASK=${EVAL_TASK_ID} \
    TBENCH_REF=${TBENCH_REF}

# The instruction the agent sees (TASK) — agent-readable by design.
COPY instruction.md /task/instruction.md

# Root-only test harness — never agent-readable (the agent must not see the tests).
COPY tests /tests
RUN chmod -R 700 /tests && chown -R root:root /tests

# Grader: run the upstream tests/test.sh, which writes /logs/verifier/reward.txt.
RUN printf '%s\n' \
  '#!/bin/bash' \
  'mkdir -p /logs/verifier' \
  'cd /app 2>/dev/null || true' \
  'bash /tests/test.sh > /logs/verifier/test.log 2>&1 || true' \
  '[ -f /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt' \
  > /grade.sh && chmod +x /grade.sh

# Entrypoint: seed TASK from the instruction if the runner didn't set it.
RUN printf '%s\n' \
  '#!/bin/bash' \
  'if [ -z "$TASK" ] && [ -f /task/instruction.md ]; then export TASK="$(cat /task/instruction.md)"; fi' \
  'exec "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
