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
git config --global --add safe.directory '*'
BASE="$(cat /tests/.base_commit)"

# ── 1. capture ──────────────────────────────────────────────────────────────
# Diff against BASE, not HEAD: an agent that committed its fix must not score 0
# for a bookkeeping reason. Pre-existing untracked files are not agent work.
git add -A 2>/dev/null
if [ -s /tests/.base_untracked ]; then
  while IFS= read -r p; do git reset -q -- "$p" 2>/dev/null; done < /tests/.base_untracked
fi
git diff --cached --binary "$BASE" > /logs/artifacts/model.patch 2>/dev/null
git reset -q 2>/dev/null

# ── 2. restore ──────────────────────────────────────────────────────────────
# `git clean -fd`, NOT -x: ignored files (node_modules, build caches) are the
# toolchain the tests need, and there is no network to reinstall them.
git reset -q --hard "$BASE" 2>/dev/null
excl=()
while IFS= read -r p; do [ -n "$p" ] && excl+=(-e "/$p"); done < /tests/.base_untracked
git clean -qfd "${excl[@]}" 2>/dev/null

# ── 3. replay + verify ──────────────────────────────────────────────────────
# patch_replay's own fallbacks. An empty patch applies nothing (reward 0).
if [ -s /logs/artifacts/model.patch ]; then
  git apply --verbose /logs/artifacts/model.patch \
    || git apply --3way /logs/artifacts/model.patch \
    || patch --fuzz=3 -p1 -i /logs/artifacts/model.patch \
    || echo "model.patch did not apply" >> /logs/artifacts/apply.log
fi
# IPv4-first name resolution for Node test suites. Kubernetes and Docker write
# `::1 localhost` into /etc/hosts; Node 18 resolves localhost to ::1 first, while
# NodeBB's test server listens on IPv4 only, so its HTTP tests die with
# ECONNREFUSED ::1:4568 and the GOLD patch scores 0 (measured on exgentic:
# 285/291 required tests, 291/291 with this set). Upstream's Modal sandbox has no
# ::1 entry, so this restores its behaviour rather than changing the tests.
# Non-Node suites ignore NODE_OPTIONS.
NODE_OPTIONS="${NODE_OPTIONS:+$NODE_OPTIONS }--dns-result-order=ipv4first" \
  bash /tests/test.sh > /logs/swe-bench-pro-verifier.log 2>&1

# test.sh's trap always writes reward.txt; a missing one means the verifier
# itself never ran, which -1 reports as "not graded" (rule 18).
[ -s /logs/verifier/reward.txt ] || echo -1 > /logs/verifier/reward.txt
exit 0
