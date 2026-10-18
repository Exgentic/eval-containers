"""Grade one AlgoTune task with Harbor's protocol (tests/test_outputs.py); README.md.

usage: grade.py <dir> <task> <n>   (reads <dir>/solver.py, writes <dir>/result.json)
"""

import importlib
import importlib.util
import json
import sys
import time
from pathlib import Path

work, name, n = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
module = importlib.import_module(f"AlgoTuneTasks.{name}.{name}")
task = next(
    c
    for c in vars(module).values()
    if isinstance(c, type)
    and c.__module__ == module.__name__
    and hasattr(c, "generate_problem")
)()


def timed(fn, problem):
    start = time.perf_counter_ns()
    fn(problem)
    return time.perf_counter_ns() - start


solver_time = reference_time = 0
valid = True
try:
    spec = importlib.util.spec_from_file_location("solver", work / "solver.py")
    solver_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(solver_module)
    solve = solver_module.Solver().solve
    for problem in [task.generate_problem(n=n, random_seed=i) for i in range(100)]:
        task.solve(problem)
        if not task.is_solution(problem, solve(problem)):
            valid = False
            break
        times = [(timed(task.solve, problem), timed(solve, problem)) for _ in range(10)]
        reference_time += min(r for r, _ in times)
        solver_time += min(s for _, s in times)
except Exception as e:  # noqa: BLE001 - any failure of the agent's code is an invalid solver
    print(f"solver failed: {e!r}")
    valid = False

speedup = reference_time / solver_time if valid and solver_time else 0.0
score = speedup if valid and speedup >= 1.0 else 1.0
result = {
    "reward": 1 - 1 / score,
    "score": score,
    "speedup": speedup,
    "valid": valid,
    "n": n,
}
(work / "result.json").write_text(json.dumps(result, indent=2))
print(json.dumps(result))
