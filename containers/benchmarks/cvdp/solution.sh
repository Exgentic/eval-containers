#!/bin/bash
# Oracle gold for cvdp (single-turn): lay out the input files plus the task's
# reference output files under /code and grade them with the task's own harness —
# the same path run_cvdp.py takes with the model's files. The public dataset
# REDACTS the reference output, so this fails loudly there; it works on a build
# whose dataset carries solutions (Si2 LBC). Runs as root; /tasks is root-only.
set -euo pipefail
T="/tasks/${EVAL_TASK_ID:-0}"
if [ ! -d "$T/output" ]; then
  echo "cvdp oracle: no reference solution in this dataset revision (redacted upstream)" >&2
  exit 1
fi
python3 - "$T" <<'PY'
import json, os, sys
t = sys.argv[1]
for name, body in json.load(open(f"{t}/task.json"))["input_context"].items():
    os.makedirs(os.path.dirname(f"/code/{name}"), exist_ok=True)
    open(f"/code/{name}", "w").write(body)
PY
cp -r "$T/output/." /code/
mkdir -p /logs/verifier
/usr/bin/python3 /tests/cvdp_grade.py "$T" /logs/verifier/services > /logs/verifier/reward.txt
