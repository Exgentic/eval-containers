# syntax=docker/dockerfile:1
# Terminal-Bench 2.0 — per-task eval overlay.
#
# Terminal-bench is the first benchmark whose per-task environment must be BUILT
# from source: no per-task upstream images exist, and each task ships its OWN
# Dockerfile with a heterogeneous base (python-3-13, ubuntu, …) plus setup. So
# the per-task build is two steps, orchestrated by build.sh:
#   1. build the task's own upstream Dockerfile (its base + setup) -> the task env
#   2. THIS file overlays the eval pipeline on that env (TASK_BASE = the env image)
# Both steps use the upstream task dir at a pinned ref as the build context.
#
# We add ONLY: the instruction (TASK), a root-only copy of the test harness, the
# grader, and the entrypoint. The gold solution is NEVER baked, and the raw tests
# are root-only (chmod 700) — the agent can read neither the answer nor the tests
# (benchmarks/RULES.md rule 9 / eval integrity; see core/oracle, solution.sh).
ARG TASK_BASE

# Parse task.yaml -> instruction.md in an isolated python stage, so we don't need
# the (heterogeneous) task base to ship python + pyyaml.
FROM python:3.13-slim AS meta
COPY task.yaml /task.yaml
RUN pip install --no-cache-dir pyyaml==6.0.2 \
 && python -c "import yaml,pathlib; pathlib.Path('/instruction.md').write_text(yaml.safe_load(open('/task.yaml')).get('instruction',''))"

FROM ${TASK_BASE}
# One label per line: the CLI detects per-task benchmarks by a `LABEL
# eval.benchmark.env="per-task"` line (benchmark::is_per_task), so it must not be
# folded into a multi-line LABEL (benchmarks/RULES.md 24f).
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="terminal-bench"
LABEL eval.benchmark.tasks="241"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/laude-institute/terminal-bench"
# EVAL_TASK_ID is consumed here (the per-task marker, benchmarks/RULES.md 24a/24f).
# TBENCH_TASK pins the real upstream task name for solution.sh: the oracle/runner
# override EVAL_TASK_ID at run time (their /tasks index convention), so the gold
# fetcher can't rely on it. TBENCH_REF is the pinned upstream commit. build.sh
# passes EVAL_TASK_ID + TBENCH_REF as build args.
ARG EVAL_TASK_ID
ARG TBENCH_REF
ENV EVAL_BENCHMARK=tbench \
    EVAL_TASK_ID=${EVAL_TASK_ID} \
    TBENCH_TASK=${EVAL_TASK_ID} \
    TBENCH_REF=${TBENCH_REF}

# The instruction the agent sees (TASK) — agent-readable by design.
COPY --from=meta /instruction.md /task/instruction.md

# Root-only test harness — never agent-readable (the agent must not see the tests).
COPY tests /tests/tests
COPY run-tests.sh /tests/run-tests.sh
RUN chmod +x /tests/run-tests.sh && chmod -R 700 /tests && chown -R root:root /tests

# Grader: run the task's upstream run-tests.sh, map pytest exit -> reward.
RUN printf '%s\n' \
  '#!/bin/bash' \
  'mkdir -p /logs/verifier' \
  'export TEST_DIR=/tests/tests' \
  'if /tests/run-tests.sh > /logs/verifier/test.log 2>&1; then' \
  '  echo 1 > /logs/verifier/reward.txt' \
  'else' \
  '  echo 0 > /logs/verifier/reward.txt' \
  'fi' > /grade.sh && chmod +x /grade.sh

# Entrypoint: seed TASK from the instruction if the runner didn't set it.
RUN printf '%s\n' \
  '#!/bin/bash' \
  'if [ -z "$TASK" ] && [ -f /task/instruction.md ]; then export TASK="$(cat /task/instruction.md)"; fi' \
  'exec "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
