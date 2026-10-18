"""Write AlgoTune's 154 tasks as /tasks/all.jsonl, one task per line.

usage: build-tasks.py <upstream source tree> <out.jsonl>

Task N is line N, tasks sorted by name. `n` is the problem size upstream picked
for each task (reports/generation.json). The prompt is the one Harbor's adapter
builds from AlgoTune's system prompt, pointing at /app/solver.py.
"""

import ast
import json
import sys
from pathlib import Path

PACKAGES = """cryptography cvxpy cython dace dask diffrax ecos faiss-cpu hdbscan highspy jax
networkx numba numpy ortools pandas pot psutil pulp pyomo python-sat pythran
scikit-learn scipy sympy torch""".split()


def function(source, name):
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.FunctionDef) and node.name == name:
            return ast.get_source_segment(source, node)
    raise SystemExit(f"no {name}()")


def prompt(description, solve, is_solution):
    return "\n".join(
        [
            "Apart from the default Python packages, you have access to the following additional packages:",
            *(f"- {p}" for p in PACKAGES),
            "\nYour objective is to define a class named `Solver` in `/app/solver.py` with a method:",
            "```python",
            "class Solver:",
            "    def solve(self, problem, **kwargs) -> Any:",
            "        # Your implementation goes here.",
            "        ...",
            "```",
            "\nIMPORTANT: Compilation time of your init function will not count towards your function's runtime.",
            "\nThis `solve` function will be the entrypoint called by the evaluation harness. Strive to align your class and method implementation as closely as possible with the desired performance criteria.",
            "For each instance, your function can run for at most 10x the reference runtime for that instance. Strive to have your implementation run as fast as possible, while returning the same output as the reference function (for the same given input). Be creative and optimize your approach!",
            "\n**GOALS:**",
            "Your primary objective is to optimize the `solve` function to run as as fast as possible, while returning the optimal solution.",
            "You will receive better scores the quicker your solution runs, and you will be penalized for exceeding the time limit or returning non-optimal solutions.",
            "\nBelow you find the description of the task you will have to solve. Read it carefully and understand what the problem is and what your solver should do.",
            "\n**TASK DESCRIPTION:**",
            f"\n{description}",
            "\nBelow is the reference implementation. Your function should run much quicker.",
            f"\n```python\n{solve}\n```",
            "\nThis function will be used to check if your solution is valid for a given problem. If it returns False, it means the solution is invalid:",
            f"\n```python\n{is_solution}\n```",
        ]
    )


src, out = Path(sys.argv[1]), Path(sys.argv[2])
sizes = json.loads((src / "reports/generation.json").read_text())
rows = []
for name in sorted(sizes):
    code = (src / "AlgoTuneTasks" / name / f"{name}.py").read_text()
    description = (src / "AlgoTuneTasks" / name / "description.txt").read_text().strip()
    rows.append(
        {
            "id": name,
            "n": sizes[name]["n"],
            "problem": prompt(
                description, function(code, "solve"), function(code, "is_solution")
            ),
        }
    )
assert len(rows) == 154, f"expected 154 tasks, got {len(rows)}"
out.write_text("".join(json.dumps(r) + "\n" for r in rows))
print(f"wrote {len(rows)} tasks")
