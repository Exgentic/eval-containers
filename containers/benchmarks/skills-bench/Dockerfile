# syntax=docker/dockerfile:1
# SkillsBench (benchflow-ai/skillsbench) — per-task eval overlay (Harbor format).
#
# Each task ships an environment/Dockerfile (the task env, heterogeneous base +
# setup), a direct instruction.md, tests/ (test.sh + test_outputs.py), and
# solution/solve.sh. No per-task upstream images exist, so the per-task build is
# two steps (build.sh): (1) build environment/Dockerfile -> the task env; (2) THIS
# file overlays the eval pipeline on it (TASK_BASE = the env image).
#
# We add ONLY: the instruction (TASK), a root-only copy of the tests, pytest for
# the grader, the grader, and the entrypoint. The gold solution is NEVER baked,
# and the tests are root-only (chmod 700) — the agent can read neither the answer
# nor the tests (benchmarks/RULES.md rule 9 / 24g; see core/oracle, solution.sh).
ARG TASK_BASE
FROM ${TASK_BASE}

# Build args that parameterize this per-task overlay (consumed below).
# EVAL_TASK_ID is the per-task marker (benchmarks/RULES.md 24a/24f). SB_TASK pins
# the real task name for solution.sh: the oracle/runner override EVAL_TASK_ID at
# run time (their /tasks index), so the gold fetcher can't rely on it. SB_REF is
# the pinned upstream commit — it pins BOTH code and data (the repo holds the
# tasks, tests, solutions, and env Dockerfiles); build.sh is its single source.
ARG EVAL_TASK_ID
ARG SB_REF

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="skills-bench"
LABEL eval.benchmark.description="SkillsBench — expert tasks across software engineering, math, cybersecurity, and more"
LABEL eval.benchmark.tasks="86"
LABEL eval.benchmark.env="per-task"
# Agent needs egress: many skills fetch data at run time (e.g. citation-management
# searches Google Scholar / PubMed), so the with-skills eval requires internet
# (rule 9 — "unless the benchmark explicitly requires it"). Label-only, as for the
# other internet=true benchmarks (gaia, assistantbench) and upstream skills-bench.
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.url="https://www.skillsbench.ai/"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2602.12670"
# Record the pinned upstream revision (rule 3/4) — driven by the same SB_REF that
# build.sh fetches from, so the label can never disagree with the baked data.
LABEL eval.benchmark.data_revision="${SB_REF}"

ENV EVAL_BENCHMARK=skills-bench \
    EVAL_TASK_ID=${EVAL_TASK_ID} \
    SB_TASK=${EVAL_TASK_ID} \
    SB_REF=${SB_REF}

# The instruction the agent sees (TASK) — agent-readable by design.
COPY instruction.md /task/instruction.md

# Skills — SkillsBench's whole point. The task's environment/skills/ is dropped at
# the agent's working directory (the inherited WORKDIR, where /run.sh runs `claude`
# without cd'ing) so claude-code auto-discovers them as project skills
# (./.claude/skills) in -p mode. Agent-READABLE by design, unlike /tests: these are
# the expert knowledge the agent is meant to leverage. Every upstream task ships
# environment/skills/.
COPY environment/skills ./.claude/skills

# Root-only tests — never agent-readable (the agent must not see the tests/answers).
COPY tests /tests
RUN chmod -R 700 /tests && chown -R root:root /tests

# pytest for the grader, installed at build time so grading needs no network.
# Upstream tests/test.sh fetches pytest via `uvx` at run time; we install it here
# instead to keep grading offline and reproducible. The task env already provides
# every other dep the tests import (it is the same env upstream runs the tests in).
RUN python3 -m pip install --no-cache-dir --break-system-packages pytest==8.4.1 pytest-json-ctrf==0.3.5 \
 || python3 -m pip install --no-cache-dir pytest==8.4.1 pytest-json-ctrf==0.3.5

# Agent output dirs (tasks write to absolute /root and /output paths).
RUN mkdir -p /root /output && chmod 777 /root /output

# Grader: run the upstream tests against the agent's result, write the reward.
# A task that emits a continuous score (civ6) drops it in scores/; otherwise the
# reward is pytest pass/fail. (benchmarks/RULES.md rule 18.)
RUN printf '%s\n' \
  '#!/bin/bash' \
  'mkdir -p /logs/verifier/scores' \
  'export PYTHONPATH="/tests/src:${PYTHONPATH:-}"' \
  'cd /root' \
  'python3 -m pytest --ctrf /logs/verifier/ctrf.json /tests/test_outputs.py -rA -v && ok=1 || ok=0' \
  'if [ -f /logs/verifier/scores/scenario_3.txt ]; then' \
  '  cat /logs/verifier/scores/scenario_3.txt > /logs/verifier/reward.txt' \
  'else' \
  '  echo "$ok" > /logs/verifier/reward.txt' \
  'fi' \
  > /grade.sh && chmod +x /grade.sh

# Entrypoint: seed TASK from the instruction if the runner didn't set it.
RUN printf '%s\n' \
  '#!/bin/bash' \
  'if [ -z "$TASK" ] && [ -f /task/instruction.md ]; then export TASK="$(cat /task/instruction.md)"; fi' \
  'exec "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
