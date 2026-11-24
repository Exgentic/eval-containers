#!/bin/bash
# Oracle for the assertllm2-verify benchmarks: derive the verdict by comparing
# the task's RTL with its design's golden, rebuilt from upstream.
#
# As root, with network: fetch the pinned upstream tree, install the emitter the
# harness used (PyVerilog 1.2.1 with upstream's patch, and Icarus Verilog as its
# preprocessor), and run the harness's own build-tasks.py on this one design.
# Its golden is the reference: the same files means "correct", any difference
# "incorrect". That checks the tasks against the golden/mutant assumption the
# labels rest on, not against the spec.
set -euo pipefail
T="/tasks/${EVAL_TASK_ID:-0}"
design=$(cat "$T/design.txt")
W=$(mktemp -d /tmp/assertllm2-oracle.XXXXXX)
chmod 700 "$W"

rev=$(python3 -c 'import json; print(json.load(open("/tasks/build/report.json"))["revision"])')
key=$(python3 -c 'import json, sys; print(json.load(open("/tasks/build/report.json"))["designs"][sys.argv[1]]["key"])' "$design")
command -v iverilog >/dev/null || {
  apt-get update -qq && apt-get install -y -qq --no-install-recommends iverilog >/dev/null
}
python3 -m venv "$W/venv"
"$W/venv/bin/pip" install -q pyverilog==1.2.1 jinja2==3.1.6 markupsafe==3.0.3 ply==3.11
mkdir "$W/src"
curl -fsSL "https://github.com/hkust-zhiyao/AssertLLM2/archive/${rev}.tar.gz" | tar -xz --strip-components=1 -C "$W/src"
P=$("$W/venv/bin/python" -c 'import os, pyverilog; print(os.path.dirname(pyverilog.__file__))')
cp "$W/src/mutation/pyverilog_change/"{ast.py,parser.py,ast_classes.txt} "$P/vparser/"
cp "$W/src/mutation/pyverilog_change/codegen.py" "$P/ast_code_generator/"

echo "$key" > "$W/designs.txt"
mkdir "$W/out"
(cd "$W" && "$W/venv/bin/python" /tests/build-tasks.py "$W/src" "$W/designs.txt" "$W/out") >/dev/null

mkdir -p /home/agent
"$W/venv/bin/python" - "$W/out/all.jsonl" "$design" <<'PY'
import json, sys
from pathlib import Path

row = next(r for r in map(json.loads, open(sys.argv[1])) if r["id"] == f"{sys.argv[2]}/golden")
golden = json.loads(row["files"])
shown = {str(p.relative_to("/app")): p.read_text() for p in Path("/app/rtl").rglob("*") if p.is_file()}
differ = sorted(f for f in set(golden) | set(shown) if golden.get(f) != shown.get(f))
answer = {
    "verdict": "incorrect" if differ else "correct",
    "explanation": f"differs from the golden RTL in {', '.join(differ)}" if differ else "identical to the golden RTL",
}
Path("/home/agent/answer.json").write_text(json.dumps(answer))
print(json.dumps(answer))
PY
chown 1002 /home/agent/answer.json 2>/dev/null || true
rm -rf "$W"
