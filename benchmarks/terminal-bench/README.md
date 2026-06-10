# terminal-bench

Terminal-Bench 2.0 — terminal tasks, built from upstream source.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | per task (`original-tasks/<task>` upstream) |
| Environment | per-task |
| Internet required | false (agent) · true (build) |
| Released | no |
| Upstream | [github.com/laude-institute/terminal-bench](https://github.com/laude-institute/terminal-bench) |

## What the agent sees

Each task is a terminal environment plus an instruction. The instruction is
extracted from the task's `task.yaml` into `/task/instruction.md` and passed to
the agent via the `TASK` env var. The agent works in `/app` (the task's working
directory) and must modify files in place.

## How it's graded

`/grade.sh` runs the task's upstream `run-tests.sh` (a pytest suite) against the
post-agent filesystem and writes `1`/`0` to `/logs/verifier/reward.txt`. The test
harness lives in a **root-only** `/tests` (chmod 700) — the agent can read neither
the tests nor any solution (benchmarks/RULES.md rule 9 / eval integrity).

## Per-task build (built from source)

Terminal-bench is the first benchmark whose per-task environment must be **built
from source**: no per-task upstream images exist, and each task ships its own
Dockerfile with a heterogeneous base (`python-3-13`, `ubuntu`, …) plus setup. So
`build.sh` builds each per-task image in two steps (benchmarks/RULES.md 24g):

1. build the task's **own** upstream Dockerfile (its base + setup) → the task env;
2. overlay our eval pipeline (`Dockerfile`, `FROM ${TASK_BASE}`) → instruction,
   root-only tests, grader, entrypoint.

Both steps use the upstream task dir at the pinned `TBENCH_REF` (in `build.sh`) as
the build context, fetched directly by the builder.

## Oracle

`solution.sh` fetches **this task's** upstream reference solution at `TBENCH_REF`
and runs it in `/app` (handles both `solution.sh` and `solution.yaml`). It is
fetched fresh at oracle run time — never baked into the agent image. Verify:

```bash
eval-containers oracle terminal-bench --task-id hello-world --local
```

## Files

- `Dockerfile` — the eval overlay (`FROM ${TASK_BASE}`); not built directly, see `build.sh`
- `build.sh` — the two-step per-task build (task env → overlay)
- `solution.sh` — oracle gold (fetches the per-task upstream solution)
- `container.Dockerfile` — single-image pin (`evals/terminal-bench-<task>--<agent>`)
- `compose.yaml` — compose file for `eval-containers run terminal-bench`
- `README.md` — this file
