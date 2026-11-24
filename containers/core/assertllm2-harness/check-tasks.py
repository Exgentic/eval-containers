"""Compile and elaborate every task as the agent will see it; keep only those that pass.

usage: check-tasks.py <in.jsonl> <tasks.txt> <out.jsonl> <diagnostics.jsonl>

Each task's files are laid out as /app would hold them, then built with every
tool a setup ships: Icarus Verilog (compile + elaborate), Verilator (lint),
Yosys (elaborate + prepare a model), and a depth-1 SymbiYosys BMC run with
the one solver shipped (smtbmc with Z3). A design whose golden fails is
dropped whole. These checks say a task can be built, not that its label is
true. The tasks that pass must be exactly tasks.txt, so a changed toolchain or
corpus fails the build instead of quietly changing the benchmark.
"""

import json
import os
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


def run(cmd, cwd, timeout=1800):
    try:
        p = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout
        )
        return p.returncode, (p.stdout + p.stderr)[-4000:]
    except subprocess.TimeoutExpired:
        return "timeout", ""


def check(row):
    with tempfile.TemporaryDirectory() as t:
        for rel, text in json.loads(row["files"]).items():
            (Path(t) / rel).parent.mkdir(parents=True, exist_ok=True)
            (Path(t) / rel).write_text(text)
        rtl = Path(t) / "rtl"
        top = row["top_module"]
        srcs = [
            line
            for line in (rtl / "files.f").read_text().split()
            if not line.startswith("+")
        ]
        reads = " ".join(f"{rtl / s}" for s in srcs)
        steps = {
            "iverilog": [
                "iverilog",
                "-g2012",
                "-s",
                top,
                "-o",
                "/dev/null",
                "-c",
                "files.f",
            ],
            "verilator": [
                "verilator",
                "--lint-only",
                "--no-timing",
                "-Wno-fatal",
                "-Wno-lint",
                "-Wno-style",
                "--top-module",
                top,
                "-f",
                "files.f",
            ],
            "yosys": [
                "yosys",
                "-q",
                "-p",
                f"read_verilog -sv -I{rtl / 'include'} {reads}; prep -top {top}",
            ],
        }
        (Path(t) / "smoke.sby").write_text(
            "[options]\nmode bmc\ndepth 1\n\n[engines]\nsmtbmc z3\n\n"
            f"[script]\nread_verilog -formal -sv -I{rtl / 'include'} {reads}\nprep -top {top}\n"
        )
        steps["sby smtbmc z3"] = ["sby", "-f", "smoke.sby"]
        results = {}
        for name, cmd in steps.items():
            start = time.monotonic()
            rc, log = run(cmd, rtl if name in ("iverilog", "verilator") else t)
            results[name] = {
                "command": cmd,
                "rc": rc,
                "seconds": round(time.monotonic() - start, 1),
                "log": "" if rc == 0 else log,
            }
    return {
        "id": row["id"],
        "ok": all(r["rc"] == 0 for r in results.values()),
        "steps": results,
    }


def main():
    src, expected, out, diag = map(Path, sys.argv[1:5])
    rows = [json.loads(line) for line in src.read_text().splitlines()]
    with ThreadPoolExecutor(os.cpu_count() or 4) as pool:
        checks = list(pool.map(check, rows))
    versions = {
        tool: run(cmd, "/")[1].strip().splitlines()[0]
        for tool, cmd in {
            "iverilog": ["iverilog", "-V"],
            "verilator": ["verilator", "--version"],
            "yosys": ["yosys", "-V"],
            "z3": ["z3", "--version"],
            "sby": ["sby", "--help"],
        }.items()
    }
    # A timeout says how fast this host is, not whether the task builds: it must
    # not decide what the benchmark contains.
    slow = [
        f"{c['id']}: {k}"
        for c in checks
        for k, v in c["steps"].items()
        if v["rc"] == "timeout"
    ]
    if slow:
        sys.exit(
            "check-tasks: timed out, so nothing is decided:\n  " + "\n  ".join(slow)
        )
    broken_golden = {
        c["id"].split("/")[0]
        for c in checks
        if c["id"].endswith("/golden") and not c["ok"]
    }
    kept = [
        r for r, c in zip(rows, checks) if c["ok"] and r["design"] not in broken_golden
    ]
    diag.write_text(
        json.dumps({"versions": versions, "dropped_designs": sorted(broken_golden)})
        + "\n"
        + "".join(json.dumps(c) + "\n" for c in checks)
    )
    for c in checks:
        if not c["ok"]:
            failed = [k for k, v in c["steps"].items() if v["rc"] != 0]
            print(f"excluded {c['id']}: {', '.join(failed)}")
    want = [
        line.strip()
        for line in expected.read_text().splitlines()
        if line.strip() and not line.startswith("#")
    ]
    got = [r["id"] for r in kept]
    if got != want:
        sys.exit(
            "check-tasks: the tasks that pass are not tasks.txt\n"
            f"  new:     {sorted(set(got) - set(want))}\n  missing: {sorted(set(want) - set(got))}\n"
            "  accepted, in order:\n" + "".join(f"    {i}\n" for i in got)
        )
    out.write_text("".join(json.dumps(r) + "\n" for r in kept))
    print(f"{len(kept)} tasks pass ({len(rows) - len(kept)} excluded)")


if __name__ == "__main__":
    main()
