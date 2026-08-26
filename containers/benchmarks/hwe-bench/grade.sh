#!/bin/bash
# Capture the candidate (agent/gold) diff, then run hwe-bench's grader.
mkdir -p /logs/verifier
cd "${HWE_REPO_DIR:-/home}" 2>/dev/null || true
git config --global --add safe.directory "${HWE_REPO_DIR:-/home}" 2>/dev/null || true
# --ignore-submodules=all: some cases' image build (prepare_script) populates
# submodules / nested build repos (e.g. .deps/riscv-fesvr with a compiled
# libfesvr.so), which git reports as dirty "Subproject commit …-dirty" hunks.
# Baking those into the candidate patch makes grade.py's `git apply` fail
# atomically ("does not exist in index"), silently dropping the real source fix.
# The agent only ever edits tracked HDL source, so submodule pointers are noise.
git diff --ignore-submodules=all > /home/fix.patch 2>/dev/null || true
python3 /tests/grade.py > /logs/verifier/reward.txt 2>>/logs/verifier/grade.log || echo 0 > /logs/verifier/reward.txt
