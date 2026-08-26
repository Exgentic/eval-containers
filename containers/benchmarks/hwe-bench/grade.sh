#!/bin/bash
# Capture the candidate (agent/gold) diff, then run hwe-bench's grader.
mkdir -p /logs/verifier
cd "${HWE_REPO_DIR:-/home}" 2>/dev/null || true
git config --global --add safe.directory "${HWE_REPO_DIR:-/home}" 2>/dev/null || true
git diff > /home/fix.patch 2>/dev/null || true
python3 /tests/grade.py > /logs/verifier/reward.txt 2>>/logs/verifier/grade.log || echo 0 > /logs/verifier/reward.txt
