#!/bin/bash
# SWE-bench Pro V2 verifier — upstream's locked protocol, in place.
#
# Upstream (v2/README.md, "The locked protocol"): the locked agent wrapper
# captures the agent's work as model.patch, and patch_replay applies it to a
# PRISTINE image and runs the unchanged tests/test.sh. We do the same three steps
# inside the agent's container:
#   1. capture — diff everything the agent changed (committed or not) against the
#      base commit into /logs/artifacts/model.patch;
#   2. restore — reset to the base commit and drop untracked files the agent
#      created, keeping the ones the base image shipped and ignored files
#      (installed dependencies);
#   3. replay  — apply model.patch with patch_replay's fallbacks, then run
#      upstream's test.sh, whose EXIT trap writes /logs/verifier/reward.txt
#      (1 iff every fail_to_pass + pass_to_pass test PASSES, else 0).
set -uo pipefail

mkdir -p /logs/verifier /logs/artifacts
REPO=/app; [ -d /app ] || REPO=/testbed
cd "$REPO" || { echo -1 > /logs/verifier/reward.txt; exit 0; }
BASE="$(cat /tests/.base_commit)"
fail() {
  echo "verifier setup failed: $*" >> /logs/artifacts/apply.log
  echo -1 > /logs/verifier/reward.txt
  exit 0
}

# The agent owns the worktree, including .git. Restore trusted configuration and
# disable Git's execution/redirection surfaces before invoking it as root.
export GIT_NO_REPLACE_OBJECTS=1 GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
rm -f .git/index.lock
rm -rf .git/hooks .git/refs/replace
install -m 600 /tests/.base_git_config .git/config || fail "restore git config"
git() { command git -c safe.directory="$REPO" -c core.fsmonitor=false -c core.hooksPath=/dev/null "$@"; }

# ── 1. capture ──────────────────────────────────────────────────────────────
# Diff against BASE, not HEAD: an agent that committed its fix must not score 0
# for a bookkeeping reason. Pre-existing untracked files are not agent work.
git add -A >> /logs/artifacts/apply.log 2>&1 || fail "git add"
if [ -s /tests/.base_untracked ]; then
  git reset -q --pathspec-from-file=/tests/.base_untracked --pathspec-file-nul \
    >> /logs/artifacts/apply.log 2>&1 || fail "exclude pristine untracked files"
fi
# /logs is a 100 MiB tmpfs in the chart. Stop below that limit so an unignored
# build artifact cannot fill it and strand the pod without a result.
git diff --cached --binary "$BASE" 2>> /logs/artifacts/apply.log \
  | head -c 94371841 > /tmp/model.patch || fail "capture patch (over 90 MiB?)"
[ "$(wc -c < /tmp/model.patch)" -le 94371840 ] || fail "captured patch exceeds 90 MiB"
mv /tmp/model.patch /logs/artifacts/model.patch || fail "store patch"
git reset -q >> /logs/artifacts/apply.log 2>&1 || fail "reset index"

# ── 2. restore ──────────────────────────────────────────────────────────────
# `git clean -fd`, NOT -x: ignored files (node_modules, build caches) are the
# toolchain the tests need, and there is no network to reinstall them.
git reset -q --hard "$BASE" >> /logs/artifacts/apply.log 2>&1 || fail "restore base commit"
git clean -qfd >> /logs/artifacts/apply.log 2>&1 || fail "clean agent files"
tar -xf /tests/.base_untracked.tar || fail "restore pristine untracked files"

# ── 3. replay + verify ──────────────────────────────────────────────────────
# patch_replay's own fallbacks. An empty patch applies nothing (reward 0).
if [ -s /logs/artifacts/model.patch ]; then
  git apply --verbose /logs/artifacts/model.patch \
    || git apply --3way /logs/artifacts/model.patch \
    || patch --fuzz=3 -p1 -i /logs/artifacts/model.patch \
    || fail "model.patch did not apply"
fi
# IPv4-first name resolution for Node test suites. Kubernetes and Docker write
# `::1 localhost` into /etc/hosts; Node 18 resolves localhost to ::1 first, while
# NodeBB's test server listens on IPv4 only, so its HTTP tests die with
# ECONNREFUSED ::1:4568 and the GOLD patch scores 0 (measured on exgentic:
# 285/291 required tests, 291/291 with this set). Upstream's Modal sandbox has no
# ::1 entry, so this restores its behaviour rather than changing the tests.
# Non-Node suites ignore NODE_OPTIONS.
NODE_OPTIONS="${NODE_OPTIONS:+$NODE_OPTIONS }--dns-result-order=ipv4first" \
  timeout -k 30 3000 bash /tests/test.sh > /logs/swe-bench-pro-verifier.log 2>&1 || true

# test.sh's trap always writes reward.txt; a missing one means the verifier
# itself never ran, which -1 reports as "not graded" (rule 18).
[ -s /logs/verifier/reward.txt ] || echo -1 > /logs/verifier/reward.txt
# /logs is in-memory and goes with the pod, so a 0 would leave no trace of why.
# Keep the verifier's output beside result.json (as terminal-bench does, a3b682ed).
mkdir -p /output/task/verifier
cp -r /logs/verifier/. /output/task/verifier/ 2>/dev/null || true
cp /logs/artifacts/model.patch /output/task/verifier/model.patch 2>/dev/null || true
cp /logs/swe-bench-pro-verifier.log /output/task/verifier/ 2>/dev/null || true
exit 0
