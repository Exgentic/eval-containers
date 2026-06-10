#!/bin/bash
# Oracle gold for terminal-bench — fetch THIS task's upstream reference solution at
# the pinned ref and run it in /app; the grader then runs the upstream pytest suite
# against the result (gold => reward 1, no-op => 0).
#
# Fetched fresh at oracle run time (never baked into the agent image), mirroring
# humaneval; the oracle runs as root with the network the sandboxed agent lacks.
# See core/oracle/README.md. TBENCH_REF / EVAL_TASK_ID are baked by the image.
#
# 233/241 tasks ship solution.sh; 8 ship solution.yaml (a list of {command:} steps).
# solution.yaml is parsed with the stdlib only, so it works even in task envs that
# lack PyYAML/pip (e.g. broken-python, whose whole task is a broken pip).
set -euo pipefail
ref="${TBENCH_REF:?TBENCH_REF not set}"
# The oracle/runner override EVAL_TASK_ID with their /tasks index, so use the real
# upstream task name baked into the image (TBENCH_TASK), falling back to EVAL_TASK_ID.
task="${TBENCH_TASK:-${EVAL_TASK_ID:?TBENCH_TASK/EVAL_TASK_ID not set}}"
base="https://raw.githubusercontent.com/laude-institute/terminal-bench/${ref}/original-tasks/${task}"

fetch() { # $1=url -> stdout, via whatever the (heterogeneous) task env provides
  if command -v curl >/dev/null 2>&1; then curl -fsSL "$1"
  elif command -v wget >/dev/null 2>&1; then wget -qO- "$1"
  else python3 -c 'import sys,urllib.request; sys.stdout.write(urllib.request.urlopen(sys.argv[1]).read().decode())' "$1"
  fi
}

cd /app
if sol=$(fetch "${base}/solution.sh" 2>/dev/null) && [ -n "${sol}" ]; then
  printf '%s\n' "${sol}" | bash
else
  # solution.yaml: extract each `- command:` (inline or block scalar), run in order.
  fetch "${base}/solution.yaml" | python3 -c '
import subprocess, sys
cmds, cur, grab = [], [], False
SIBS = ("min_timeout_sec:", "max_timeout_sec:", "block:", "append_enter:")
for raw in sys.stdin.read().splitlines():
    if raw.lstrip().startswith("#"):
        continue
    if raw.startswith("- "):
        if cur:
            cmds.append(" ".join(cur)); cur = []
        rest = raw[2:]
        if rest.strip().startswith("command:"):
            v = rest.split("command:", 1)[1].strip()
            if v:
                cmds.append(v); grab = False
            else:
                grab = True
        continue
    s = raw.strip()
    if grab and s:
        if any(s.startswith(k) for k in SIBS):
            grab = False
        else:
            cur.append(s)
if cur:
    cmds.append(" ".join(cur))
for c in cmds:
    subprocess.run(c, shell=True, check=True)
'
fi
