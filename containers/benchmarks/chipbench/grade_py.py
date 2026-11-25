"""Grade one Python reference model against a ChipBench task, as upstream does.

    python3 /app/grade_py.py <task-dir> <dut.py> <work-dir>   -> prints "1.0" or "0.0"

Checks, in upstream's order (Ref Model Gen/gen.py `run_test_detailed`): the code
is non-empty, defines `class TopModule`, and compiles; then upstream's
Tool_Box/crosslang_verify (the paper's Heterogeneous Test Engine) compiles the
golden ref.sv with Verilator and compares it with the Python model on random
stimuli — 1,000 tests for combinational designs, 500 cycles after a 20-cycle
reset for sequential ones, outputs masked during reset:

    python3 /opt/crosslang_verify/main.py ref.sv --dut-py dut.py -w work

Pass iff it exits 0. One deviation, closing a hole: upstream's comparison skips
any output port missing from the model's returned dict, so a TopModule whose
eval() returns {} passes every task the checker can build. The model's source is
followed, in the same `dut.py` (the checker copies only that file), by a wrapper
class that fills each output
port the model omits, named by the checker's own port parser, with a value no
output can equal. An honest model returns every port, so its verdict is
unchanged. The model's Python is executed by that simulation, so it
runs as `nobody` in a fresh scratch dir, with every written file capped at
FSIZE_CAP (RLIMIT_FSIZE) and the whole check killed after CHECK_TIMEOUT.
work-dir keeps the checker log and verdict.json.
"""

import json
import os
import re
import resource
import shutil
import subprocess
import sys
import tempfile

CROSSLANG = os.environ.get("CHIPBENCH_CROSSLANG", "/opt/crosslang_verify/main.py")
FSIZE_CAP = 16 * 1024 * 1024
CHECK_TIMEOUT = 600  # Verilator build (~1 min) + the checker's own 300 s run cap
NOBODY = 65534
WRAPPER = """

_Model = TopModule
_OUTPUTS = {outputs!r}


class TopModule:
    def __init__(self):
        self._m = _Model()

    def eval(self, inputs):
        out = self._m.eval(inputs)
        out = dict(out) if isinstance(out, dict) else {{}}
        for name in _OUTPUTS:
            out.setdefault(name, "missing")
        return out
"""


def output_ports(ref_sv):
    """The output names crosslang_verify itself extracts from ref.sv."""
    sys.path.insert(0, os.path.dirname(CROSSLANG))
    from tools.extract_ports import extract_ports_from_verilog

    return [name for name, _ in extract_ports_from_verilog(ref_sv)[1]]


def contained():
    resource.setrlimit(resource.RLIMIT_FSIZE, (FSIZE_CAP, FSIZE_CAP))
    if os.geteuid() == 0:
        os.setgroups([])
        os.setgid(NOBODY)
        os.setuid(NOBODY)


def verdict_of(log, rc):
    if rc == 124:
        return "timeout"
    if "FAILED: Verilator RefModule" in log:
        return "ref-compile-error"
    if re.search(r"FAILED: (make|linking)", log):
        return "build-error"
    return "pass" if rc == 0 else "mismatch"


def grade(task_dir, dut, work):
    os.makedirs(work, exist_ok=True)
    code = open(dut, errors="replace").read()
    log_path = os.path.join(work, "crosslang.log")
    if not code.strip():
        verdict = "empty"
    elif "class TopModule" not in code:
        verdict = "no-topmodule"
    else:
        try:
            compile(code, "dut.py", "exec")
            verdict = None
        except SyntaxError:
            verdict = "syntax-error"
    if verdict is None:
        scratch = tempfile.mkdtemp(prefix="chipbench-ref-")
        try:
            shutil.copyfile(f"{task_dir}/ref.sv", f"{scratch}/ref.sv")
            with open(f"{scratch}/dut.py", "w") as f:
                f.write(
                    code + WRAPPER.format(outputs=output_ports(f"{task_dir}/ref.sv"))
                )
            if os.geteuid() == 0:
                for root, dirs, files in os.walk(scratch):
                    for name in dirs + files:
                        os.chown(os.path.join(root, name), NOBODY, NOBODY)
                os.chown(scratch, NOBODY, NOBODY)
            with open(log_path, "w") as log:
                rc = subprocess.run(
                    [
                        "timeout",
                        str(CHECK_TIMEOUT),
                        "python3",
                        CROSSLANG,
                        "ref.sv",
                        "--dut-py",
                        "dut.py",
                        "-w",
                        "work",
                    ],
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    cwd=scratch,
                    preexec_fn=contained,
                ).returncode
        finally:
            shutil.rmtree(scratch, ignore_errors=True)
        with open(log_path, errors="replace") as f:
            verdict = verdict_of(f.read(), rc)
    with open(os.path.join(work, "verdict.json"), "w") as f:
        json.dump({"passfail": verdict}, f)
    return verdict


if __name__ == "__main__":
    v = grade(*sys.argv[1:4])
    print("1.0" if v == "pass" else "0.0")
    print(f"[chipbench] crosslang_verify verdict: {v}", file=sys.stderr)
