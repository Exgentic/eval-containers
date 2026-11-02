"""Turn upstream's verdict on one BackendBench operator into the reward.

usage: grade.py <dir> <op>   (reads <dir>/out/full_results.json, writes <dir>/result.json)

`<dir>/out` is what upstream's CLI wrote for the operator: one row per OpInfo
sample it ran. The reward is upstream's per-operator correctness — 1.0 when every
sample matches PyTorch, else 0.0.
"""

import json
import sys
from pathlib import Path

work, op = Path(sys.argv[1]), sys.argv[2]
path = work / "out" / "full_results.json"
rows = json.loads(path.read_text()) if path.is_file() else []
tests = [r for r in rows if r.get("test_type") == "correctness"]
passed = sum(r["is_correct"] for r in tests)
result = {
    "reward": 1.0 if tests and passed == len(tests) else 0.0,
    "op": op,
    "tests": len(tests),
    "passed": passed,
    "correctness_rate": passed / len(tests) if tests else 0.0,
}
if not tests:
    result["error"] = (
        "upstream ran no test: the kernel did not load, or the run failed (backendbench.log)"
    )
(work / "result.json").write_text(json.dumps(result, indent=2))
print(json.dumps(result))
