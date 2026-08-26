# syntax=docker/dockerfile:1
# DeepSWE (Datacurve v1.1) — per-task eval overlay.
#
# Source: github.com/datacurve-ai/deep-swe, dataset `datacurve/deep-swe-1-1`.
# 113 long-horizon tasks (TypeScript, Go, Python, JavaScript, Rust) drawn from
# active OSS repos. Each task ships instruction.md, environment/Dockerfile,
# tests/ (test.sh + grader.py + test.patch + config.json), and solution/.
#
# Upstream publishes a pinned per-task image (task.toml `[environment].docker_image`),
# so build.sh resolves it and passes it as TASK_BASE and THIS file overlays the
# eval pipeline on it. The base already holds the repo at base_commit with origin
# removed and future history gc'd ("git time-travel"), so the reference solution
# cannot leak from git history.
#
# We add ONLY: the instruction (TASK), a root-only copy of the verifier
# (tests/ = test.sh, grader.py, test.patch, config.json), the grader, and the
# entrypoint. The gold solution is NEVER baked — the oracle's solution.sh fetches
# it fresh (rules 5, 9, 20a).
#
# v1.1 deviation, deliberate: upstream grades in a SEPARATE pristine container
# (task.toml `[verifier].environment_mode = "separate"`), collecting the agent's
# work as /logs/artifacts/model.patch via `[[verifier.collect]]`. This repo's
# three-phase flow grades IN-PLACE after the agent (rule 12), the same deviation
# swe-bench documents. We preserve the semantics that matter: /grade.sh runs the
# collect command to produce model.patch, and grader.py `prepare` resets exactly
# the files that patch touches back to base_commit before re-applying it, so
# grading is not contaminated by uncommitted agent scratch state. Recorded in
# AUDIT.md.
ARG TASK_BASE
FROM ${TASK_BASE}
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="deepswe"
LABEL eval.benchmark.description="DeepSWE v1.1 - long-horizon software engineering on active open-source repositories"
LABEL eval.benchmark.tasks="113"
LABEL eval.benchmark.env="per-task"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/datacurve-ai/deep-swe"
# No paper: DeepSWE is a dataset + leaderboard release, not a publication. The
# label is required by the benchmark PR template, so it is set explicitly to "n/a"
# rather than omitted, so a reader can tell "none exists" from "nobody checked".
LABEL eval.benchmark.paper="n/a"
LABEL eval.benchmark.upstream_base="public.ecr.aws/d3j8x8q7/swe-bench-202605 (per-task tag from task.toml)"

# EVAL_TASK_ID is consumed here (the per-task marker, rules 24a/24f).
# DEEPSWE_TASK pins the real task name for solution.sh: the oracle/runner
# override EVAL_TASK_ID at run time (their /tasks index), so the gold fetcher
# can't rely on it (rule 24i). DEEPSWE_REF is the pinned dataset commit, which
# also backs the data_revision label. build.sh passes all three as build args.
ARG EVAL_TASK_ID
ARG DEEPSWE_REF
LABEL eval.benchmark.data_revision="${DEEPSWE_REF}"
ENV EVAL_BENCHMARK=deepswe \
    EVAL_TASK_ID=${EVAL_TASK_ID} \
    DEEPSWE_TASK=${EVAL_TASK_ID} \
    DEEPSWE_REF=${DEEPSWE_REF}

# The instruction the agent sees (TASK) — agent-readable by design.
COPY instruction.md /task/instruction.md

# Root-only verifier — never agent-readable. tests/ holds the hidden tests
# (test.patch), the whitelist of graded node ids (config.json), and the grader.
# An agent that could read config.json would know exactly which tests are graded.
COPY tests /tests
RUN chmod -R 700 /tests && chown -R root:root /tests

# Grader: reproduce upstream's verifier sequence in-place.
#   1. collect — the task.toml [[verifier.collect]] command: diff the agent's work
#      against base_commit into /logs/artifacts/model.patch.
#   2. test.sh — upstream's own entrypoint: grader.py prepare (reset the touched
#      files to base, re-apply model.patch, apply test.patch), run the suites,
#      grader.py grade -> /logs/verifier/reward.json.
#   3. bridge reward.json -> reward.txt, the framework's contract (rule 18).
#      reward.json.reward is already binary 0/1; -1 stays the crash sentinel.
#
# Our own files come from the named `pipeline` build context, not the main one:
# the main context is the UPSTREAM task directory (a git URL), which of course
# holds no files of ours. build.sh binds pipeline= this benchmark's directory.
# Entrypoint: seeds TASK from the instruction if the runner didn't set it.
COPY --from=pipeline grade.sh /grade.sh
COPY --from=pipeline entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
