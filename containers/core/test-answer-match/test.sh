#!/bin/bash
# Shared ZeroBench grader (→ /grade.sh in the agent-facing benchmark). The agent
# stdout is a TRANSCRIPT, so score_answer.py's CLI anchors to the LAST
# `FINAL ANSWER:` sentinel line (not the native "last {...}", which is correct
# only for a single model completion) and matches EXPECTED_ANSWER case-
# insensitively. Fail-closed: pre-seed 0, and the scorer re-seeds 0 before
# deciding, so a missing sentinel / empty stdout / empty EXPECTED_ANSWER stays 0.
# Shared with the native harness, which imports extract_answer/is_correct.
mkdir -p /logs/verifier
echo 0 > /logs/verifier/reward.txt
python3 /score_answer.py "$EXPECTED_ANSWER"
