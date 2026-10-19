#!/bin/bash
# Oracle for algotune. Upstream ships no reference speedup solution, but its
# results/ holds the solvers frontier models wrote, with each one's measured
# speedup in reports/agent_summary.json. The gold is the fastest of them for this
# task, fetched fresh at the pinned revision — never baked into the image.
set -euo pipefail
rev=dff9914c10800c7a031c9e8c3d4d1c8cd1b38906
tmp=$(mktemp -d)
curl -sfL "https://github.com/oripress/AlgoTune/archive/$rev.tar.gz" | tar xz -C "$tmp"
python3 - "$tmp/AlgoTune-$rev" "$(cat "/tasks/${EVAL_TASK_ID:-0}/id.txt")" <<'PY'
import json, re, shutil, sys
from pathlib import Path

src, task = Path(sys.argv[1]), sys.argv[2]
names = dict(re.findall(r'"([^"]+)":\s*"([^"]+)"', (src / "html/batch_html_generator.py").read_text()))
best = []
for model, entry in json.loads((src / "reports/agent_summary.json").read_text())[task].items():
    try:
        speedup = float(entry["final_speedup"])
    except ValueError:
        continue
    solver = src / "results" / names.get(model, "") / task / "solver.py"
    if solver.is_file():
        best.append((speedup, solver))
if not best:
    raise SystemExit(f"no upstream solver for {task}")
speedup, solver = max(best)
shutil.copy(solver, "/app/solver.py")
print(f"{solver.parent.parent.name}: {speedup}x upstream")
PY
