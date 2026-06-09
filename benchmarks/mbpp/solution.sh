#!/bin/bash
# Oracle for mbpp: emit the canonical MBPP `code`. The grader runs it against the
# task's asserts. id.txt holds the MBPP task_id (e.g. "11"), which the row lookup
# matches. The field is root-only in the image, so it is fetched from the pinned
# dataset parquet. Output goes to stdout; the oracle redirects it.
set -euo pipefail
python3 - "${EVAL_TASK_ID:-0}" <<'PY'
import sys, os, urllib.request
import pyarrow.parquet as pq

# --- per-benchmark config ---
NAME = "mbpp"
URL  = ("https://huggingface.co/datasets/google-research-datasets/mbpp/"
        "resolve/refs%2Fconvert%2Fparquet/full/test/0000.parquet")
COLS = ["code"]                          # concatenated, in order, to form the solution
OVERRIDES = {}                           # task_id -> completion, for genuine grader edge cases

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
