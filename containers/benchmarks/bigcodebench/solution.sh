#!/bin/bash
# Oracle for BigCodeBench: emit a complete canonical program = code_prompt (imports
# + signature) + canonical_solution (body), both from the same pinned dataset row.
# The grader runs it against the task's tests. The fields are root-only in the
# image, so they are fetched. Output goes to stdout; the oracle redirects it.
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
NAME = "bigcodebench"
REV  = "b74c0d0bf70d2c0bc459be537895cca163007f1a"
URL  = ("https://huggingface.co/datasets/bigcode/bigcodebench/resolve/"
        f"{REV}/data/v0.1.4-00000-of-00001.parquet")
COLS = ["code_prompt", "canonical_solution"]   # concatenated, in order, to form the solution
OVERRIDES = {}                                 # task_id -> completion, for genuine grader edge cases

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
