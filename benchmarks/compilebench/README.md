# compilebench

CompileBench - compile open source projects

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 15 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/QuesmaOrg/CompileBench](https://github.com/QuesmaOrg/CompileBench) |
| Paper | — |
| Dataset revision | `66e27468505706643088b79f8efad6260c274dc5` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/0/problem.txt)"" The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

See `/tests/test.sh` in the built image for the scoring logic. Grading runs inside the per-task image after the agent exits.

This benchmark uses `env=per-task`: the Dockerfile takes a `DOCK_TASK_ID` build-arg and pulls a per-task upstream base image for each task.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run compilebench`
- `README.md` — this file
