#!/bin/bash
# Oracle gold for chipbench: the golden RefModule renamed to TopModule, graded by
# the same grade_sv.py the native harness uses. ref.sv is compiled alongside the
# answer, so the gold carries only what ref.sv/test.sv do not already define:
# the RefModule block itself plus any submodule that only the prompt supplies
# (the hierarchical gen_not_self_contain tasks). Runs as root; /tasks is root-only.
set -euo pipefail
T="/tasks/${EVAL_TASK_ID:-0}"
mkdir -p /logs/verifier/sim
python3 - "$T" > /logs/verifier/sim/TopModule.sv <<'PY'
import re, sys
t = sys.argv[1]
MOD = re.compile(r"^[ \t]*module[ \t]+(\w+).*?^[ \t]*endmodule\b", re.S | re.M)
def modules(text):
    return {m.group(1): m.group(0) for m in MOD.finditer(text)}
ref, test = open(f"{t}/ref.sv").read(), open(f"{t}/test.sv").read()
prompt_code = "\n".join(re.findall(r"```[^\n]*\n(.*?)```", open(f"{t}/prompt.txt").read(), re.S))
have = set(modules(ref)) | set(modules(test))
# The gen_cpu_ip / cpu_top golden code uses `define macros that only test.sv
# declares — and test.sv compiles AFTER the answer — so the gold restates them.
for line in test.splitlines():
    if line.lstrip().startswith("`define"):
        print(line)
print(re.sub(r"\bmodule(\s+)RefModule\b", r"module\1TopModule", modules(ref)["RefModule"], count=1))
for name, body in modules(prompt_code).items():
    if name not in have and name != "TopModule":
        print(body)
PY
python3 /app/grade_sv.py "$T" /logs/verifier/sim/TopModule.sv /logs/verifier/sim > /logs/verifier/reward.txt
