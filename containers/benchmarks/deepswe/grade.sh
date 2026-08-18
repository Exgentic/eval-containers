#!/bin/bash
# DeepSWE v1.1 verifier — reproduce upstream's separate-container verifier
# sequence in-place, then bridge its reward.json to the framework's reward.txt.
#
# Upstream runs the verifier in a pristine container (task.toml
# [verifier].environment_mode = "separate"), collecting the agent's work as
# /logs/artifacts/model.patch via [[verifier.collect]]. This repo grades in-place
# after the agent (rule 12) — the same deviation swe-bench documents. What makes
# it sound: grader.py `prepare` resets exactly the files model.patch touches back
# to base_commit before re-applying it, and then applies the hidden test.patch.
# So the graded tree is base + model.patch + test.patch regardless of whatever
# scratch state the agent left behind.
#
# Reward: grader.py writes /logs/verifier/reward.json with a binary 0/1 `reward`
# (1 iff there are fail-to-pass tests, all pass, and no pass-to-pass regression).
# We copy that to reward.txt (rule 18). -1 is the crash sentinel that upstream's
# own test.sh trap writes when the verifier itself breaks (rule 18 allows -1).
set -uo pipefail

mkdir -p /logs/verifier /logs/artifacts

BASE="$(python3 -c 'import json;print(json.load(open("/tests/config.json"))["base_commit"])' 2>/dev/null)"

# ── 1. collect (the task.toml [[verifier.collect]] command) ─────────────────
# Diff the agent's work against base_commit. Upstream diffs <base>..HEAD, which
# assumes the agent committed. Agents often edit without committing, so stage
# everything into a commit first — an uncommitted-but-correct fix must not score
# 0 for a bookkeeping reason. If the agent already committed, this is a no-op.
cd /app || { echo -1 > /logs/verifier/reward.txt; exit 6; }
git config --global --add safe.directory /app
if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
  git -c user.name=eval -c user.email=eval@local add -A 2>/dev/null
  git -c user.name=eval -c user.email=eval@local commit -q -m "agent work" 2>/dev/null
fi
git diff --binary "${BASE}" HEAD > /logs/artifacts/model.patch 2>/dev/null

# Restore the pristine base tree before grading. Upstream grades in a SEPARATE
# container where the agent's edits never happened, so its `prepare` only needs to
# reset the files model.patch MODIFIES. In-place that is not enough: a patch that
# ADDS a file (here fastapi/middleware/methods.py) fails to re-apply with
# "already exists in working directory", scoring a correct solution 0 with
# apply_failed=1. Hard-resetting to base_commit and clearing untracked files
# reproduces the pristine precondition prepare assumes — the patch we just
# collected is the sole record of the agent's work from here on.
# NOTE: `git clean -fd` (NOT -x). Ignored files must survive: the base image
# installs dependencies into the work tree (node_modules for the TS/JS tasks,
# build caches elsewhere) and they are gitignored, so -x would delete the very
# toolchain the suites need and there is no network to reinstall it.
git reset -q --hard "${BASE}" 2>/dev/null
git clean -qfd 2>/dev/null

# ── 2. run upstream's verifier entrypoint ───────────────────────────────────
# test.sh owns prepare -> suites -> grade and writes reward.json (or, via its own
# EXIT trap, the reward.txt=-1 crash sentinel).
#
# The log MUST live outside /logs/verifier: test.sh globs /logs/verifier/*.log to
# echo raw suite output into its stdout, so logging there makes it read the file
# it is presently writing — the cat never terminates and `grader.py grade` is
# never reached (observed: reward.txt=-1 with no reward.json). /logs/deepswe-verifier.log
# keeps the full transcript while leaving test.sh's glob to the suite logs it means.
bash /tests/test.sh > /logs/deepswe-verifier.log 2>&1

# ── 3. bridge reward.json -> reward.txt (rule 18) ───────────────────────────
if [ -f /logs/verifier/reward.json ]; then
  python3 -c 'import json;print(json.load(open("/logs/verifier/reward.json"))["reward"])' \
    > /logs/verifier/reward.txt 2>/dev/null \
    || echo 0 > /logs/verifier/reward.txt
fi
# No reward at all => the verifier never produced a verdict; -1 says "not graded"
# rather than silently reporting a 0 the agent didn't earn.
[ -f /logs/verifier/reward.txt ] || echo -1 > /logs/verifier/reward.txt
exit 0
