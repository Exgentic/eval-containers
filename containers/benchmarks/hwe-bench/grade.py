"""hwe-bench grader — HWE-Bench's OWN deterministic, simulation-based method.

Each case carries a self-contained `tb_script` that generates its testbench,
compiles the repo's RTL with the baked Verilator, and prints
`TEST: <name> ... PASS|FAIL|SKIP` markers between HWE_BENCH_RESULTS_START/END.
Grading: reset the repo to the buggy base commit, apply the candidate diff
(agent's edits, captured by grade.sh; empty => no-op), run tb_script, and resolve
iff every fail-to-pass test PASSES. Strict resolved@1 = the leaderboard metric;
no fractional reward. Standard library only — no dataset package, fully offline.
"""

import json
import os
import re
import subprocess

LOG = open("/logs/verifier/grade.log", "a")


def sh(cmd):
    subprocess.run(cmd, shell=True, executable="/bin/bash", stdout=LOG, stderr=LOG)


try:
    cfg = json.load(open("/tasks/0/config.json"))
    repo = os.environ.get("HWE_REPO_DIR", "/home") or "/home"
    base = cfg["base"]["sha"]
    f2p = set((cfg.get("f2p_tests") or {}).keys())

    os.chdir(repo)
    sh("git config --global --add safe.directory " + repo)
    sh("git reset --hard")
    sh("git checkout -f " + base)
    # Candidate diff (agent/gold); empty => no-op. -3way tolerates minor context drift.
    sh("git apply -v --3way /home/fix.patch || git apply -v /home/fix.patch || true")

    proc = subprocess.run(
        "bash /tests/tb_script.sh",
        shell=True,
        executable="/bin/bash",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    LOG.write(proc.stdout)

    status = {}
    for m in re.finditer(
        r"^TEST:\s+(\S+)\s+\.\.\.\s+(PASS|FAIL|SKIP)\s*$", proc.stdout, re.M
    ):
        status[m.group(1)] = m.group(2)

    resolved = bool(f2p) and all(status.get(t) == "PASS" for t in f2p)
    print(1 if resolved else 0)
except Exception as exc:  # noqa: BLE001 - any failure is a non-resolution
    LOG.write("grade error: %s\n" % exc)
    print(0)
