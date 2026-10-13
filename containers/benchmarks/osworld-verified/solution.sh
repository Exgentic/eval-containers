#!/bin/bash
# Oracle: upstream publishes no reference solutions, so a task's gold is the
# actions one passing agent run sent to the desktop, replayed at that run's own
# pace (solutions/<upstream id>.jsonl, written by solution-from-calls.py). A
# task has one once an agent has solved it.
set -euo pipefail
python3 - "/oracle-solutions/$(cat "/tasks/${EVAL_TASK_ID}/id.txt").jsonl" <<'PY'
import json, os, sys, time

import requests

for step in map(json.loads, open(sys.argv[1])):
    time.sleep(step["wait"])
    requests.post("http://%s:5000/act" % os.environ["EVAL_DESKTOP_HOST"], json={"action": step["action"]})
PY
