"""swe-bench-pro grader — the benchmark's OWN per-instance method.

Ports upstream's create_entryscript (scaleapi/SWE-bench_Pro-os): reset the repo to
base, apply the candidate diff, check out the test files (the last line of
before_repo_set_cmd), run the per-instance run_script.sh, parse its output with the
per-instance parser.py, and resolve iff every fail_to_pass + pass_to_pass test
PASSES. Standard library only — no swebench package (it doesn't know these repos).
"""

import ast
import json
import os
import subprocess

LOG = open("/logs/verifier/grade.log", "a")


def sh(cmd):
    subprocess.run(cmd, shell=True, executable="/bin/bash", stdout=LOG, stderr=LOG)


def as_list(v):
    return v if isinstance(v, list) else ast.literal_eval(v)


try:
    cfg = json.load(open("/tasks/0/config.json"))
    base = cfg["base_commit"]
    before = cfg["before_repo_set_cmd"].strip().splitlines()[-1]
    selected = ",".join(as_list(cfg["selected_test_files_to_run"]))
    f2p = set(as_list(cfg["fail_to_pass"]))
    p2p = set(as_list(cfg["pass_to_pass"]))

    os.chdir("/app")
    sh("git config --global --add safe.directory /app")
    sh("git reset --hard " + base)
    sh("git checkout " + base)
    sh(
        "git apply -v /workspace/patch.diff || true"
    )  # candidate (agent/gold); empty => no-op
    sh(before)  # checks out the test files
    sh(
        "bash /tests/run_script.sh "
        + selected
        + " > /workspace/stdout.log 2> /workspace/stderr.log"
    )
    sh(
        "python3 /tests/parser.py /workspace/stdout.log /workspace/stderr.log /workspace/output.json"
    )

    out = json.load(open("/workspace/output.json"))
    passed = {t["name"] for t in out.get("tests", []) if t.get("status") == "PASSED"}
    print(1 if f2p and (f2p | p2p) <= passed else 0)
except Exception as exc:  # noqa: BLE001 - any failure is a non-resolution
    LOG.write("grade error: %s\n" % exc)
    print(0)
