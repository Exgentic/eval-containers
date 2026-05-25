# APPS (Measuring Coding Challenge Competence)
# 5000 test tasks. Data: HuggingFace codeparrot/apps
# Agent writes a Python solution. Test runs it against JSON-encoded I/O test cases.

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="apps"
LABEL eval.benchmark.description="APPS - Python coding problems across 3 difficulty tiers"
LABEL eval.benchmark.tasks="5000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="21e74ddf8de1a21436da12e3e653065c5213e9d1"
LABEL eval.benchmark.url="https://github.com/hendrycks/apps"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2105.09938"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="21e74ddf8de1a21436da12e3e653065c5213e9d1"

# Single-JSONL task layout. Agent never sees /tasks/all.jsonl (root-only).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, pyarrow, os, json
tables = []
for part in (0, 1):
    url = f'https://huggingface.co/datasets/codeparrot/apps/resolve/refs%2Fconvert%2Fparquet/all/test/000{part}.parquet'
    urllib.request.urlretrieve(url, f'/tmp/data_{part}.parquet')
    tables.append(pq.read_table(f'/tmp/data_{part}.parquet'))
t = pyarrow.concat_tables(tables)
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        starter = str(t['starter_code'][i]) if t['starter_code'][i] else ''
        io_raw = str(t['input_output'][i]) if t['input_output'][i] else ''
        fn_name = ''
        if io_raw:
            try:
                io_obj = json.loads(io_raw)
                fn_name = io_obj.get('fn_name', '') or ''
            except Exception:
                pass
        row = {
            'id': str(t['problem_id'][i]),
            'problem': str(t['question'][i]),
            'difficulty': str(t['difficulty'][i]),
            'starter_code': starter,
            'input_output': io_raw,
            'fn_name': fn_name,
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=apps

# Test: run agent's Python solution against decoded I/O test cases.
# Handles both stdio mode and functional mode (when fn_name is set).
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$EVAL_TASK_ID"
SOLUTION=$(cat /output/agent/stdout.log 2>/dev/null)

if [ -z "$SOLUTION" ] || [ ! -s "$TASK_DIR/input_output.txt" ]; then
  echo 0 > /logs/verifier/reward.txt
  exit 0
fi

TMPDIR=$(mktemp -d)
printf '%s' "$SOLUTION" > $TMPDIR/sol.py
cp "$TASK_DIR/input_output.txt" $TMPDIR/io.json
cp "$TASK_DIR/fn_name.txt" $TMPDIR/fn.txt
export APPS_TMPDIR="$TMPDIR"

# Runner compares actual vs expected for each I/O case and prints PASSED/TOTAL
timeout 300 python3 <<'PYEOF' > $TMPDIR/result.txt 2>/dev/null
import io, json, sys, contextlib, os
TMP = os.environ["APPS_TMPDIR"]
sol_src = open(os.path.join(TMP, "sol.py")).read()
io_obj = json.loads(open(os.path.join(TMP, "io.json")).read())
fn_name = open(os.path.join(TMP, "fn.txt")).read().strip()
inputs = io_obj.get("inputs", [])
outputs = io_obj.get("outputs", [])
passed = 0
total = 0

def norm(x):
    if isinstance(x, list):
        return "\n".join(str(i) for i in x)
    return str(x)

for inp, exp in zip(inputs, outputs):
    total += 1
    try:
        if fn_name:
            ns = {}
            exec(sol_src, ns)
            fn = ns.get(fn_name)
            if fn is None and "Solution" in ns:
                fn = getattr(ns["Solution"](), fn_name, None)
            if fn is None:
                continue
            args = inp if isinstance(inp, list) else [inp]
            actual = fn(*args)
            expected = exp[0] if isinstance(exp, list) and len(exp) == 1 else exp
            if actual == expected:
                passed += 1
        else:
            stdin_text = inp if isinstance(inp, str) else "\n".join(str(x) for x in inp)
            buf_out = io.StringIO()
            buf_err = io.StringIO()
            old_stdin = sys.stdin
            sys.stdin = io.StringIO(stdin_text)
            try:
                with contextlib.redirect_stdout(buf_out), contextlib.redirect_stderr(buf_err):
                    ns = {"__name__": "__main__"}
                    exec(sol_src, ns)
            finally:
                sys.stdin = old_stdin
            actual = buf_out.getvalue().strip()
            expected = norm(exp).strip() if not isinstance(exp, str) else exp.strip()
            if actual == expected:
                passed += 1
            elif "".join(actual.split()) == "".join(expected.split()):
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
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  STARTER=""
  if [ -s "/tasks/$EVAL_TASK_ID/starter_code.txt" ]; then
    STARTER="

Starter code:
$(cat /tasks/$EVAL_TASK_ID/starter_code.txt)"
  fi
  export TASK="Solve this Python coding problem. Print ONLY the complete Python source code, nothing else. If starter code is given, use it.$STARTER

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
