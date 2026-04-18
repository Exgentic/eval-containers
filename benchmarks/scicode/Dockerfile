# SciCode (scientific research coding benchmark)
# 65 main problems, 288 subproblems. Data: HuggingFace SciCode1/SciCode
# Numeric ground-truth tensors: Google Drive (~1GB test_data.h5).
# Agent writes Python code for each sub-step. Test asserts output equals loaded targets.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="scicode"
LABEL dock.benchmark.description="SciCode - scientific research coding across 16 subdomains"
LABEL dock.benchmark.tasks="65"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="4510f6a6aa27c43fad7b43da2c59602a86e88480"
LABEL dock.benchmark.url="https://github.com/scicode-bench/SciCode"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2407.13168"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="4510f6a6aa27c43fad7b43da2c59602a86e88480"

# Runtime deps for test execution (tests use numpy/scipy/h5py and call the loader)
RUN pip install --no-cache-dir numpy==2.0.2 scipy==1.14.1 h5py==3.12.1 sympy==1.13.3

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/.
# sub_steps and general_tests are stored as JSON strings so that when the
# materializer writes them to <field>.txt the file contains valid JSON text.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/SciCode1/SciCode/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        sub = t['sub_steps'][i].as_py() or []
        gen_tests = list(t['general_tests'][i].as_py() or [])
        row = {
            'id': str(t['problem_id'][i]),
            'problem': str(t['problem_description_main'][i] or ''),
            'background': str(t['problem_background_main'][i] or ''),
            'io': str(t['problem_io'][i] or ''),
            'dependencies': str(t['required_dependencies'][i] or ''),
            'sub_steps': json.dumps(sub),
            'general_tests': json.dumps(gen_tests),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

# Download test_data.h5 from Google Drive (file id 17G_k65N_6yFFZ2O-jQH00Lh6iaw3z-AW, ~1GB)
# Uses the "confirm" token bypass required for large files.
RUN mkdir -p /tests/data && python3 <<'PYEOF'
import urllib.request, urllib.parse, http.cookiejar, re, sys, os
FILE_ID = "17G_k65N_6yFFZ2O-jQH00Lh6iaw3z-AW"
OUT = "/tests/data/test_data.h5"
cj = http.cookiejar.CookieJar()
opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cj))
opener.addheaders = [("User-Agent", "Mozilla/5.0")]
URL = f"https://drive.usercontent.google.com/download?id={FILE_ID}&export=download&confirm=t"
with opener.open(URL) as r, open(OUT, "wb") as f:
    while True:
        chunk = r.read(1 << 20)
        if not chunk:
            break
        f.write(chunk)
size = os.path.getsize(OUT)
print(f"Downloaded {size} bytes")
if size < 1_000_000:
    # Likely an HTML error page; bail out loudly.
    sys.exit("test_data.h5 download too small; Google Drive may have changed tokens")
PYEOF
RUN chmod 600 /tasks/all.jsonl /tests/data/test_data.h5

WORKDIR /app
ENV DOCK_BENCHMARK=scicode

# Test: combine agent code with all sub-step test cases and run against test_data.h5
# Scoring: fraction of subproblems passed for this main problem.
RUN cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$DOCK_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)

if [ -z "$SOLUTION" ]; then
  echo 0 > /logs/verifier/reward.txt
  exit 0
fi

TMPDIR=$(mktemp -d)
printf '%s' "$SOLUTION" > $TMPDIR/sol.py
cp "$TASK_DIR/sub_steps.txt" $TMPDIR/sub_steps.json
cp "$TASK_DIR/dependencies.txt" $TMPDIR/deps.txt
cp "$TASK_DIR/id.txt" $TMPDIR/id.txt
export SC_TMP="$TMPDIR"

timeout 600 python3 <<'PYEOF' > $TMPDIR/result.txt 2>/dev/null
import json, os, sys, h5py, scipy, numpy as np

TMP = os.environ["SC_TMP"]
H5PY_FILE = "/tests/data/test_data.h5"

# Copied from scicode.parse.parse so the benchmark is self-contained.
def process_hdf5_list(group):
    return [group[k][()] for k in group.keys()]

def process_hdf5_sparse_matrix(group):
    data = group['data'][()]
    shape = tuple(group['shape'][()])
    if 'row' in group and 'col' in group:
        return scipy.sparse.coo_matrix((data, (group['row'][()], group['col'][()])), shape=shape)
    if 'blocksize' in group:
        return scipy.sparse.bsr_matrix((data, group['indices'][()], group['indptr'][()]),
                                        shape=shape, blocksize=tuple(group['blocksize'][()]))
    return scipy.sparse.csr_matrix((data, group['indices'][()], group['indptr'][()]), shape=shape)

def process_hdf5_dict(group):
    out = {}
    for key, obj in group.items():
        if isinstance(obj, h5py.Group):
            out[key] = process_hdf5_sparse_matrix(obj['sparse_matrix'])
        elif isinstance(obj[()], bytes):
            out[key] = obj[()].decode('utf-8')
        else:
            try:
                out[float(key)] = obj[()]
            except ValueError:
                out[key] = obj[()]
    return out

def process_hdf5_datagroup(group):
    for key in group.keys():
        if key == "list":
            return process_hdf5_list(group[key])
        if key == "sparse_matrix":
            return process_hdf5_sparse_matrix(group[key])
        return process_hdf5_dict(group)

def process_hdf5_to_tuple(step_id, test_num, h5py_file=H5PY_FILE):
    data_lst = []
    with h5py.File(h5py_file, 'r') as f:
        for test_id in range(test_num):
            group_path = f'{step_id}/test{test_id + 1}'
            g = f[group_path]
            if isinstance(g, h5py.Group):
                keys = list(g.keys())
                if len(keys) == 1:
                    sub = g[keys[0]]
                    if isinstance(sub, h5py.Dataset):
                        v = sub[()]
                        data_lst.append(v.decode('utf-8') if isinstance(v, bytes) else v)
                    else:
                        data_lst.append(process_hdf5_datagroup(sub))
                else:
                    var_lst = []
                    for k in keys:
                        sub = g[k]
                        if isinstance(sub, h5py.Dataset):
                            v = sub[()]
                            var_lst.append(v.decode('utf-8') if isinstance(v, bytes) else v)
                        else:
                            var_lst.append(process_hdf5_datagroup(sub))
                    data_lst.append(tuple(var_lst))
    return data_lst

sol_src = open(os.path.join(TMP, "sol.py")).read()
deps = open(os.path.join(TMP, "deps.txt")).read()
sub_steps = json.loads(open(os.path.join(TMP, "sub_steps.json")).read())

passed = 0
total = 0
for step in sub_steps:
    step_id = step.get("step_number") or ""
    tests = step.get("test_cases") or []
    if not tests:
        continue
    total += 1
    try:
        ns = {"process_hdf5_to_tuple": process_hdf5_to_tuple}
        exec(deps, ns)
        exec(sol_src, ns)
        targets = process_hdf5_to_tuple(step_id, len(tests))
        ok = True
        for ti, tc in enumerate(tests):
            local_ns = dict(ns)
            local_ns["target"] = targets[ti]
            try:
                exec(tc, local_ns)
            except Exception:
                ok = False
                break
        if ok:
            passed += 1
    except Exception:
        continue

print(f"{passed}/{total}")
PYEOF

RESULT=$(cat $TMPDIR/result.txt 2>/dev/null)
if [ -z "$RESULT" ]; then
  echo 0 > /logs/verifier/reward.txt
else
  PASSED=${RESULT%/*}
  TOTAL=${RESULT#*/}
  if [ "$TOTAL" -gt 0 ] 2>/dev/null; then
    python3 -c "print($PASSED / $TOTAL)" > /logs/verifier/reward.txt
  else
    echo 0 > /logs/verifier/reward.txt
  fi
fi
rm -rf $TMPDIR
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  BG=""
  [ -s "/tasks/$DOCK_TASK_ID/background.txt" ] && BG="

Background:
$(cat /tasks/$DOCK_TASK_ID/background.txt)"
  SUBS=$(python3 -c "
import json
sub = json.loads(open('/tasks/$DOCK_TASK_ID/sub_steps.txt').read())
for s in sub:
    print(f\"Step {s.get('step_number','')}:\")
    print(s.get('step_description_prompt',''))
    print(s.get('function_header',''))
    print(s.get('return_line',''))
    print()
")
  export TASK="Solve this scientific research coding problem. Implement ALL sub-step functions in a single Python file. Use the given function headers and return lines exactly. Print ONLY the complete Python code (including the required imports), nothing else.$BG

Problem:
$(cat /tasks/$DOCK_TASK_ID/problem.txt)

I/O spec:
$(cat /tasks/$DOCK_TASK_ID/io.txt)

Required imports:
$(cat /tasks/$DOCK_TASK_ID/dependencies.txt)

Sub-steps:
$SUBS"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
