"""Flatten a pinned ChipBench checkout into sequential per-task dirs (rules 15/16).

    /tasks/<n>/id.txt      upstream id, "<split>/<problem>" (e.g. gen_self_contain/Prob000_Four-to-one_multiplexer)
    /tasks/<n>/task.json   {id, track, split, problem}
    /tasks/<n>/prompt.txt  upstream <problem>_prompt.txt, verbatim (what the model sees)
    /tasks/<n>/ref.sv      golden RefModule (hidden; compiled with the answer at grade)
    /tasks/<n>/test.sv     VerilogEval-style testbench `tb` (hidden)
    /tasks/<n>/system.txt  reference-model tasks only: upstream gen_python_prompt.txt

Ids 0-222 are the Verilog tracks (generation, zero-shot and one-shot
debugging). Ids 223-267 are the Python reference-model track (paper Table 3):
the 45 generation problems again, in the same order, with the user message
rewritten as upstream's `Ref Model Gen/gen.py` does (Verilog -> Python) and
graded against the same golden ref.sv by upstream's Tool_Box/crosslang_verify.

Task order is fixed: the splits in SPLITS order, problems sorted by name inside
each split. Problems are enumerated from the *_prompt.txt files, NOT from
upstream's problems.txt — at the pinned commit dataset_not_self_contain and
dataset_cpu_ip list only 1 of their 6 / 9 problems there.
"""

import json
import os
import shutil
import sys

src, out = sys.argv[1], sys.argv[2]

# (split id, upstream dir, track)
SPLITS = [
    ("gen_self_contain", "Verilog Gen/dataset_self_contain", "gen"),
    ("gen_not_self_contain", "Verilog Gen/dataset_not_self_contain", "gen"),
    ("gen_cpu_ip", "Verilog Gen/dataset_cpu_ip", "gen"),
] + [
    (
        f"debug_{shot}_{kind}",
        f"Verilog Debugging/dataset_debug_{shot}_{kind}",
        f"debug_{shot}",
    )
    for shot in ("zero_shot", "one_shot")
    for kind in ("arithmetic", "assignment", "state_machine", "timing")
]
EXPECTED = {
    "gen_self_contain": 30,
    "gen_not_self_contain": 6,
    "gen_cpu_ip": 9,
    "debug_zero_shot_arithmetic": 24,
    "debug_zero_shot_assignment": 30,
    "debug_zero_shot_state_machine": 6,
    "debug_zero_shot_timing": 29,
    "debug_one_shot_arithmetic": 24,
    "debug_one_shot_assignment": 30,
    "debug_one_shot_state_machine": 6,
    "debug_one_shot_timing": 29,
}

n = 0
for split, rel, track in SPLITS:
    d = os.path.join(src, rel)
    problems = sorted(
        f[: -len("_prompt.txt")] for f in os.listdir(d) if f.endswith("_prompt.txt")
    )
    assert (
        len(problems) == EXPECTED[split]
    ), f"{split}: expected {EXPECTED[split]}, got {len(problems)}"
    for p in problems:
        base = os.path.join(out, str(n))
        os.makedirs(base)
        for suffix, name in (
            ("_prompt.txt", "prompt.txt"),
            ("_ref.sv", "ref.sv"),
            ("_test.sv", "test.sv"),
        ):
            shutil.copyfile(os.path.join(d, p + suffix), os.path.join(base, name))
        uid = f"{split}/{p}"
        with open(os.path.join(base, "id.txt"), "w") as f:
            f.write(uid)
        with open(os.path.join(base, "task.json"), "w") as f:
            json.dump({"id": uid, "track": track, "split": split, "problem": p}, f)
        n += 1

# Python reference-model track over the generation problems (gen.py's prompt).
system = open(os.path.join(src, "Ref Model Gen/gen_python_prompt.txt")).read()
for split, rel, track in SPLITS[:3]:
    d = os.path.join(src, rel)
    for p in sorted(
        f[: -len("_prompt.txt")] for f in os.listdir(d) if f.endswith("_prompt.txt")
    ):
        base = os.path.join(out, str(n))
        os.makedirs(base)
        user = open(os.path.join(d, p + "_prompt.txt")).read()
        for a, b in (
            ("SystemVerilog", "Python"),
            ("systemverilog", "python"),
            ("Verilog", "Python"),
            ("verilog", "python"),
        ):
            user = user.replace(a, b)
        with open(os.path.join(base, "prompt.txt"), "w") as f:
            f.write(user)
        with open(os.path.join(base, "system.txt"), "w") as f:
            f.write(system)
        shutil.copyfile(os.path.join(d, p + "_ref.sv"), os.path.join(base, "ref.sv"))
        rsplit = "refmodel_python_" + split[len("gen_") :]
        uid = f"{rsplit}/{p}"
        with open(os.path.join(base, "id.txt"), "w") as f:
            f.write(uid)
        with open(os.path.join(base, "task.json"), "w") as f:
            json.dump(
                {"id": uid, "track": "refmodel_python", "split": rsplit, "problem": p},
                f,
            )
        n += 1

print(f"materialized {n} tasks")
assert n == 268, f"expected 268 tasks, got {n}"
