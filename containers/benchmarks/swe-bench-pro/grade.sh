#!/bin/bash
# Capture the candidate (agent/gold) diff, then run swe-bench-pro's grader.
mkdir -p /logs/verifier /workspace
cd /app 2>/dev/null || true
git config --global --add safe.directory /app 2>/dev/null || true
git diff > /workspace/patch.diff 2>/dev/null || true
python3 /tests/grade.py > /logs/verifier/reward.txt 2>>/logs/verifier/grade.log || echo 0 > /logs/verifier/reward.txt
