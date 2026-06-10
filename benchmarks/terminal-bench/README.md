# terminal-bench

Terminal-Bench 2.1 (Harbor) — terminal tasks, built from upstream source.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 89 (`tasks/<task>` upstream) |
| Environment | per-task |
| Internet required | false (agent) · true (build) |
| Released | no |
| Upstream | [github.com/harbor-framework/terminal-bench-2-1](https://github.com/harbor-framework/terminal-bench-2-1) |
| Leaderboard | [tbench.ai/leaderboard/terminal-bench/2.1](https://www.tbench.ai/leaderboard/terminal-bench/2.1) |

## What the agent sees

Each task is a terminal environment plus an instruction. The instruction is the
task's `instruction.md`, copied to `/task/instruction.md` and passed to the agent
via the `TASK` env var. The agent works in `/app` (the env's workdir) and must
modify files in place.

## How it's graded

`/grade.sh` runs the task's upstream `tests/test.sh` (which installs pytest and
runs `tests/test_outputs.py`, writing `1`/`0` to `/logs/verifier/reward.txt`). The
test harness lives in a **root-only** `/tests` (chmod 700) — the agent can read
neither the tests nor any solution (benchmarks/RULES.md rule 9 / eval integrity).

## Per-task build (built from source)

No per-task upstream images exist, and each task ships its own
`environment/Dockerfile` (heterogeneous base + setup). So `build.sh` builds each
per-task image in two steps (benchmarks/RULES.md 24g):

1. build the task's **own** `environment/Dockerfile` → the task env;
2. overlay our eval pipeline (`Dockerfile`, `FROM ${TASK_BASE}`) → instruction,
   root-only tests, grader, entrypoint.

Both steps fetch the upstream task dir at the pinned `TBENCH_REF` (in `build.sh`)
directly from the builder — no local checkout.

## Oracle

`solution.sh` fetches **this task's** upstream `solution/solve.sh` at `TBENCH_REF`
and runs it in `/app`. It is fetched fresh at oracle run time — never baked into
the agent image. Verify:

```bash
eval-containers oracle terminal-bench --task-id build-cython-ext --local
```

## Files

- `Dockerfile` — the eval overlay (`FROM ${TASK_BASE}`); not built directly, see `build.sh`
- `build.sh` — the two-step per-task build (task env → overlay)
- `solution.sh` — oracle gold (fetches the per-task upstream `solve.sh`)
- `container.Dockerfile` — single-image pin (`evals/terminal-bench-<task>--<agent>`)
- `compose.yaml` — compose file for `eval-containers run terminal-bench`
- `README.md` — this file
