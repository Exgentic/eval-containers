# syntax=docker/dockerfile:1
# SWE-bench Pro V2 — per-task eval overlay.
#
# Source: github.com/scaleapi/SWE-bench_Pro-os tag v2.0.0 (v2/tasks/<id>), the
# validated public split: 642 tasks across 11 repos (Python, Go, JS/TS). Same
# data as HF `ScaleAI/SWE-bench_Pro` config `default`; HARD-51 is a subset of it.
#
# build.sh resolves the task's published base (task.toml
# `[environment].docker_image` = ghcr.io/scaleapi/swe-bench_pro-v2:<id>, repo at
# /app at base_commit with a sanitised git history) and passes it as TASK_BASE;
# THIS file overlays the eval pipeline on it.
#
# We add ONLY: the instruction (TASK), a root-only copy of upstream's verifier
# (tests/ = test.sh, run_script.sh, parser.py, config.json, test_patch.patch),
# the grader and the entrypoint. The gold solution is NEVER baked — the oracle's
# solution.sh fetches it fresh (rules 5, 9, 20a).
#
# Deviation, deliberate: upstream's authoritative score comes from a SECOND
# Harbor run (patch_replay) that applies the captured model.patch to a pristine
# image and runs the unchanged verifier. This repo grades IN-PLACE after the
# agent (rule 12), like deepswe and swe-bench: /grade.sh captures model.patch the
# way upstream's locked agents do, restores the tree to the base commit, re-applies
# the patch the way patch_replay does, then runs upstream's test.sh unchanged.
# Recorded in AUDIT.md.
ARG TASK_BASE
FROM ${TASK_BASE}
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="swe-bench-pro"
LABEL eval.benchmark.description="SWE-bench Pro V2 - 642 validated professional-grade software engineering tasks across 11 repositories"
LABEL eval.benchmark.tasks="642"
LABEL eval.benchmark.env="per-task"
# Upstream: [agent] network_mode = "no-network" on all 642 tasks.
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.url="https://github.com/scaleapi/SWE-bench_Pro-os/tree/v2.0.0/v2"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2509.16941"
LABEL eval.benchmark.upstream_base="ghcr.io/scaleapi/swe-bench_pro-v2 (per-task tag from task.toml)"
# Every per-task base is a single linux/amd64 manifest (upstream README), so
# build.sh pins --platform linux/amd64 and the release skips arm64.
LABEL eval.platforms="linux/amd64"

# EVAL_TASK_ID is consumed here (the per-task marker, rules 24a/24f).
# SBP_TASK pins the real instance id for solution.sh: the oracle/runner override
# EVAL_TASK_ID at run time (rule 24i). SBP_REF is the pinned upstream commit
# (tag v2.0.0), which also backs the data_revision label.
ARG EVAL_TASK_ID
ARG SBP_REF
LABEL eval.benchmark.data_revision="${SBP_REF}"
ENV EVAL_BENCHMARK=swe-bench-pro \
    EVAL_TASK_ID=${EVAL_TASK_ID} \
    SBP_TASK=${EVAL_TASK_ID} \
    SBP_REF=${SBP_REF}

# The instruction the agent sees (TASK) — agent-readable by design. It is
# upstream's own text (PR description, requirements, new interfaces).
COPY instruction.md /task/instruction.md

# Root-only verifier — never agent-readable. config.json names the graded tests
# and test_patch.patch holds the hidden tests.
COPY tests /tests
# Record the pristine tree so grading can restore it: the base commit, local Git
# config, and an archive of the untracked files the base image already carries.
# The NUL-separated list also handles spaces and glob characters literally.
RUN cd /app 2>/dev/null || cd /testbed && \
    git config --global --add safe.directory '*' && \
    git rev-parse HEAD > /tests/.base_commit && \
    cp .git/config /tests/.base_git_config && \
    git ls-files -z --others --exclude-standard > /tests/.base_untracked && \
    tar --null -T /tests/.base_untracked -cf /tests/.base_untracked.tar && \
    chmod -R 700 /tests && chown -R root:root /tests

COPY --from=pipeline grade.sh /grade.sh
COPY --from=pipeline entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
