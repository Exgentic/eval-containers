"""Grade one AlgoTune task as Harbor's adapter does (tests/test_outputs.py).

usage: grade.py <dir> <task> <n>   (reads <dir>/solver.py, writes <dir>/result.json)

Runs as the grader uid because it executes the agent's code. 100 problems at
size n with seeds 0-99; per problem one warm-up of each function, the solver's
output checked with is_solution, then 10 interleaved timings, keeping the
minimum. Speedup is the summed reference minimum over the summed solver minimum,
and AlgoTune's score is 1.0 when the solver is invalid or slower. The reward,
1 - 1/score, is 0 at the floor and rises toward 1; a run's mean reward converts
back to AlgoTune's harmonic-mean score as 1 / (1 - mean).
"""

import os

for var in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ.setdefault(var, "8")

import importlib  # noqa: E402
import importlib.util  # noqa: E402
import json  # noqa: E402
import sys  # noqa: E402
import time  # noqa: E402
from pathlib import Path  # noqa: E402

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
