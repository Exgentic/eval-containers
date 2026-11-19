"""Grade one TopModule answer against a ChipBench task, exactly as upstream does.

    python3 /app/grade_sv.py <task-dir> <answer.sv> <work-dir>   -> prints "1.0" or "0.0"

Compile + run: upstream Makefile.in (VerilogEval harness) —
    iverilog -Wall -Winfloop -Wno-timescale -g2012 -s tb -o <bin> <answer.sv> <test.sv> <ref.sv>
    timeout 30 <bin>           (exit 124 appends "TIMEOUT")
both into ONE log. Verdict: upstream scripts/sv-iv-analyze, which scans that log
line by line and passes the sample ('.') only when no error/warning class
matched and the testbench printed "Mismatches: 0 in N samples". Its post-pass
source scan ('r'/'R') only labels failures, so it cannot change the verdict and
is folded into "fail".
"""

import json
import os
import re
import subprocess
import sys

MISMATCH = re.compile(r"^Mismatches: (\d+) in \d+ samples$")
# sv-iv-analyze, in its order: (substring, failure code); the first hit wins.
BREAKS = [
    ("syntax error", "S"),
    ("error: This assignment requires an explicit cast", "e"),
    ("error: Sized numeric constant must have a size greater than zero", "0"),
    ("warning: always_comb process has no sensitivities", "n"),
    ("found no sensitivities so it will never trigger", "n"),
    ("is declared here as wire", "w"),
    ("Unknown module type", "m"),
    ("Unable to bind wire/reg/memory `clk'", "c"),
    ("TIMEOUT", "T"),
]


def classify(log):
    error_c = error_p = no_mismatch = False
    for line in log.splitlines(keepends=True):
        if "syntax error" in line:
            return "S"
        if "error" in line:
            error_c = True
        for needle, code in BREAKS[1:]:
            if needle in line:
                return code
        if "Unable to bind wire/reg" in line:
            error_p = True
        m = MISMATCH.match(line.rstrip("\n"))
        if m and int(m.group(1)) == 0:
            no_mismatch = True
    if error_p:
        return "p"
    if error_c:
        return "C"
    return "." if no_mismatch else "R"


def grade(task_dir, answer, work):
    os.makedirs(work, exist_ok=True)
    binary, log_path = os.path.join(work, "sim"), os.path.join(work, "sv-iv-test.log")
    with open(log_path, "w") as log:
        subprocess.run(
            [
                "iverilog",
                "-Wall",
                "-Winfloop",
                "-Wno-timescale",
                "-g2012",
                "-s",
                "tb",
                "-o",
                binary,
                answer,
                f"{task_dir}/test.sv",
                f"{task_dir}/ref.sv",
            ],
            stdout=log,
            stderr=subprocess.STDOUT,
            cwd=work,
        )
    with open(log_path, "a") as log:
        if os.path.exists(binary):
            rc = subprocess.run(
                ["timeout", "30", binary],
                stdout=log,
                stderr=subprocess.STDOUT,
                cwd=work,
            ).returncode
            if rc == 124:
                log.write("TIMEOUT\n")
    code = classify(open(log_path, errors="replace").read())
    with open(os.path.join(work, "verdict.json"), "w") as f:
        json.dump({"passfail": code}, f)
    return code


if __name__ == "__main__":
    code = grade(*sys.argv[1:4])
    print("1.0" if code == "." else "0.0")
    print(f"[chipbench] sv-iv-analyze verdict: {code}", file=sys.stderr)
