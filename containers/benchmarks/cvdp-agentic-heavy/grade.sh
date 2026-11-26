#!/bin/bash
# Grade the agent's /code workspace with the task's own CVDP harness.
mkdir -p /logs/verifier
/usr/bin/python3 /tests/cvdp_grade.py "/tasks/${EVAL_TASK_ID:-0}" /logs/verifier/services \
  > /logs/verifier/reward.txt 2>>/logs/verifier/grade.log || echo 0.0 > /logs/verifier/reward.txt
# /logs is in-memory; keep the per-service pytest logs with the result.
mkdir -p /output/task && cp -r /logs/verifier /output/task/verifier 2>/dev/null || true
