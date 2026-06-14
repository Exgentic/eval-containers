#!/usr/bin/env bash
# Gold solution for swe-lancer.
#
# setup_expensify.yml re-introduces the bug with `patch -p1 < bug_reintroduce.patch`
# (when revert_command.txt is empty). Upstream's run_tests.yml documents the
# reference fix as the reverse — "run the tests on the gold patch":
#   patch -p1 -R < bug_reintroduce.patch
# which restores the upstream-fixed tree the task's test.py was written against.
#
# The patch is fetched fresh from the pinned upstream commit, NOT read from the
# image: /app/tests is root-only so the agent can't reach the gold/tests (rule 7).
# Mounted at oracle time only — never COPY'd into the agent image. Uses the baked
# ISSUE_ID/PREP_REF (EVAL_TASK_ID is overridden to 0 at oracle runtime).
set -euo pipefail
issue="${ISSUE_ID:?ISSUE_ID not set}"
ref="${PREP_REF:?PREP_REF not set}"
url="https://raw.githubusercontent.com/openai/preparedness/${ref}/project/swelancer/issues/${issue}/bug_reintroduce.patch"
cd /app/expensify
curl -fsSL --retry 3 --retry-delay 1 "${url}" | patch -p1 -R
echo "[swe-lancer] applied gold (reversed bug_reintroduce.patch) for ${issue}"
