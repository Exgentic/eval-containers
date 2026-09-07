#!/bin/bash
# HANDBOOK.md grader (benchmark CMD; runs as root in the three-phase flow).
#
# The MCP servers persist each service's state to /data/<svc>/final.json on every
# write tool call, so by the time the agent stops the gradable state is already on
# disk — no export_state call is needed. Grading is just running the verifier over
# /workdir + the service state. sop_verifier.py writes the reward to
# /logs/verifier/reward.txt (rule 18) and a detailed report to /output/task/.
#
# Verifier deps (openpyxl / pdfplumber / python-docx / pandas) are baked into the
# image, so this is network-independent.
python3 /app/sop_verifier.py
