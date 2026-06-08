# terminal-bench

Terminal-Bench 2.0 - terminal tasks

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 89 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/laude-institute/terminal-bench](https://github.com/laude-institute/terminal-bench) |
| Paper | — |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "$(cat "$f")"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

See `/grade.sh` in the built image for the scoring logic. Grading runs inside the per-task image after the agent exits.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/laude-institute/terminal-bench/${EVAL_TASK_ID}:2.0`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run terminal-bench`
- `README.md` — this file
