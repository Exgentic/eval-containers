#!/bin/bash
# Oracle gold for chipbench: the golden RefModule renamed to TopModule, graded by
# the same grade_sv.py the native harness uses. ref.sv is compiled alongside the
# answer, so the gold carries only what ref.sv/test.sv do not already define:
# the RefModule block itself plus any submodule that only the prompt supplies
# (the hierarchical gen_not_self_contain tasks). Runs as root; /tasks is root-only.
#
# Python reference-model tasks have no upstream gold (the repo ships no Python
# models of its problems), so the two below are hand-written from the golden
# RTL: a combinational and a sequential design, covering both checker paths.
# Any other reference-model task has no gold and fails loudly. Before the gold,
# an answer that reports no outputs from outside eval() (it rebinds json.dumps,
# which the checker's runner prints through) must score 0.0.
set -euo pipefail
T="/tasks/${EVAL_TASK_ID:-0}"
mkdir -p /logs/verifier/sim
if grep -q '"track": "refmodel_python"' "$T/task.json"; then
  case "$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["problem"])' "$T/task.json")" in
    Prob000_Four-to-one_multiplexer)
      cat > /logs/verifier/sim/dut.py <<'GOLD'
class TopModule:
    def __init__(self):
        pass

    def eval(self, inputs: dict) -> dict:
        d = [inputs.get(f"d{i}", 0) & 0x3 for i in range(4)]
        return {"mux_out": d[inputs.get("sel", 0) & 0x3]}
GOLD
      ;;
    Prob001_continuous_input_sequence_detect)
      cat > /logs/verifier/sim/dut.py <<'GOLD'
class TopModule:
    def __init__(self):
        self.a_tem = 0
        self.match = 0

    def eval(self, inputs: dict) -> dict:
        if not inputs.get("rst_n", 1):
            self.a_tem, self.match = 0, 0
        else:
            self.match = int(self.a_tem == 0b0111_0001)
            self.a_tem = ((self.a_tem << 1) | (inputs.get("a", 0) & 1)) & 0xFF
        return {"match": self.match}
GOLD
      ;;
    *)
      echo "chipbench oracle: no hand-written Python gold for $(cat "$T/id.txt")" >&2
      exit 1
      ;;
  esac
  mkdir -p /logs/verifier/no-outputs
  cat > /logs/verifier/no-outputs/dut.py <<'CHEAT'
import json
_d = json.dumps
json.dumps = lambda x, *a, **k: _d([{} for _ in x])
class TopModule:
    def eval(self, inputs): return {}
CHEAT
  if [ "$(python3 /app/grade_py.py "$T" /logs/verifier/no-outputs/dut.py /logs/verifier/no-outputs)" != "0.0" ]; then
    echo "chipbench oracle: an answer reporting no outputs passed $(cat "$T/id.txt")" >&2
    exit 1
  fi
  python3 /app/grade_py.py "$T" /logs/verifier/sim/dut.py /logs/verifier/sim > /logs/verifier/reward.txt
  exit 0
fi
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
