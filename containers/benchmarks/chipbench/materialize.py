"""Flatten a pinned ChipBench checkout into sequential per-task dirs (rules 15/16).

    /tasks/<n>/id.txt      upstream id, "<split>/<problem>" (e.g. gen_self_contain/Prob000_Four-to-one_multiplexer)
    /tasks/<n>/task.json   {id, track, split, problem}
    /tasks/<n>/prompt.txt  upstream <problem>_prompt.txt, verbatim (what the model sees)
    /tasks/<n>/ref.sv      golden RefModule (hidden; compiled with the answer at grade)
    /tasks/<n>/test.sv     VerilogEval-style testbench `tb` (hidden)

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

print(f"materialized {n} tasks")
assert n == 223, f"expected 223 tasks, got {n}"
