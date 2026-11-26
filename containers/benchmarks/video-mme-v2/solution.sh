#!/bin/bash
# Oracle gold: predict each question's dataset answer, in the line format the
# harness prints. The oracle redirects this stdout to /output/agent/stdout.log,
# which /grade.sh reads. The questions are the image's own (/tasks is baked per
# video, so the task identity is baked too — rule 24i). Runs as root.
set -euo pipefail
jq -c '{question_id, prediction: .answer}' /tasks/questions.jsonl
