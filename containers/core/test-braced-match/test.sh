#!/bin/bash
# Shared braced-answer grader (→ /grade.sh in agent-facing ZeroBench-style
# benchmarks). Extract the last {...} from the agent's stdout and match it
# case-insensitively to EXPECTED_ANSWER. Fail-closed: pre-seed 0, and the scorer
# itself re-seeds 0 before deciding, so a missing/empty stdout or empty
# EXPECTED_ANSWER stays 0. Single source of truth: score_braced.py (the same
# logic the native harness imports).
mkdir -p /logs/verifier
echo 0 > /logs/verifier/reward.txt
python3 /score_braced.py "$EXPECTED_ANSWER"
