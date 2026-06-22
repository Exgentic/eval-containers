#!/bin/bash
# Oracle for humanevalplus: emit the canonical_solution (function body). The grader
# splices it onto the prompt + EvalPlus harness and runs check(entry_point). For
# 163/164 tasks the dataset's canonical_solution passes. The lone exception is
# HumanEval/32 (find_zero): its plus-test asserts `_poly(*candidate(*inp), inp) <=
# 0.0001` where _poly takes exactly 2 args, so find_zero must return a 1-element
# iterable; the scalar canonical solution raises TypeError. The only body that
# satisfies it for every input yields sum([])==0 — hence the OVERRIDE below.
set -euo pipefail
python3 - "${EVAL_TASK_ID:-0}" <<'PY'
import sys, os, urllib.request
try:
    import pyarrow.parquet as pq
except ModuleNotFoundError:
    # The python-slim runtime (#197) ships no pyarrow. solution.sh is oracle-only
    # and mounted at run time (never baked), so install it here, not in the image.
    import subprocess
    subprocess.run([sys.executable, '-m', 'pip', 'install', '--quiet', '--no-cache-dir', 'pyarrow'], check=True)
    import pyarrow.parquet as pq

# --- per-benchmark config ---
NAME = "humanevalplus"
URL  = ("https://huggingface.co/datasets/evalplus/humanevalplus/resolve/"
        "refs%2Fconvert%2Fparquet/default/test/0000.parquet")
COLS = ["canonical_solution"]                  # concatenated, in order, to form the solution
OVERRIDES = {"HumanEval/32": "\n    return ([],)\n"}  # genuine grader edge cases

# --- shared: fetch pinned parquet, resolve this task's row, emit the reference ---
tid = sys.argv[1]
path = f"/tmp/oracle-{NAME}.parquet"
if not os.path.exists(path):
    urllib.request.urlretrieve(URL, path)
t = pq.read_table(path)

row = int(tid)                                       # line-index materialization
id_path = f"/tasks/{tid}/id.txt"
if os.path.exists(id_path):                          # prefer the task's real id when present
    want = open(id_path).read().strip()
    if not (0 <= row < len(t)) or str(t["task_id"][row].as_py()) != want:
        row = [str(x) for x in t["task_id"].to_pylist()].index(want)

task_id = str(t["task_id"][row].as_py())
if task_id in OVERRIDES:
    sys.stdout.write(OVERRIDES[task_id])
else:
    sys.stdout.write("".join(str(t[c][row].as_py()) for c in COLS))
PY
