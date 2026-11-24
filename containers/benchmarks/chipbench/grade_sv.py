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

The simulation executes the answer, so it is contained (the verdict rules above
are unchanged): it runs in a fresh scratch directory, as `nobody` when the
grader is root (test.sv and ref.sv are compiled into the binary, so it needs no
other file), and every file it writes — the log included — is capped at
FSIZE_CAP by RLIMIT_FSIZE; a run that hits the cap gets SIGXFSZ, prints no
Mismatches line and fails as 'R'. work-dir keeps only sv-iv-test.log,
verdict.json and, when it is at most VCD_KEEP, the testbench's wave.vcd.
"""

import json
import os
import re
import resource
import shutil
import subprocess
import sys
import tempfile

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
FSIZE_CAP = 16 * 1024 * 1024
VCD_KEEP = 1024 * 1024
NOBODY = 65534


def classify(lines):
    error_c = error_p = no_mismatch = False
    for line in lines:
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


def contained():
    resource.setrlimit(resource.RLIMIT_FSIZE, (FSIZE_CAP, FSIZE_CAP))
    if os.geteuid() == 0:
        os.setgroups([])
        os.setgid(NOBODY)
        os.setuid(NOBODY)


def grade(task_dir, answer, work):
    os.makedirs(work, exist_ok=True)
    log_path = os.path.join(work, "sv-iv-test.log")
    scratch = tempfile.mkdtemp(prefix="chipbench-sim-")
    try:
        binary = os.path.join(scratch, "sim")
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
                cwd=scratch,
                preexec_fn=lambda: resource.setrlimit(
                    resource.RLIMIT_FSIZE, (FSIZE_CAP, FSIZE_CAP)
                ),
            )
        if os.path.exists(binary):
            if os.geteuid() == 0:
                os.chown(scratch, NOBODY, NOBODY)
            with open(log_path, "a") as log:
                rc = subprocess.run(
                    ["timeout", "30", binary],
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    cwd=scratch,
                    preexec_fn=contained,
                ).returncode
                if rc == 124:
                    log.write("TIMEOUT\n")
        vcd = os.path.join(scratch, "wave.vcd")
        if os.path.isfile(vcd) and os.path.getsize(vcd) <= VCD_KEEP:
            shutil.copyfile(vcd, os.path.join(work, "wave.vcd"))
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    with open(log_path, errors="replace") as f:
        code = classify(f)
    with open(os.path.join(work, "verdict.json"), "w") as f:
        json.dump({"passfail": code}, f)
    return code


if __name__ == "__main__":
    code = grade(*sys.argv[1:4])
    print("1.0" if code == "." else "0.0")
    print(f"[chipbench] sv-iv-analyze verdict: {code}", file=sys.stderr)
