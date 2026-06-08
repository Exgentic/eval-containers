# swe-bench

SWE-bench Verified - software engineering

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/princeton-nlp/SWE-bench](https://github.com/princeton-nlp/SWE-bench) |
| Paper | [paper](https://arxiv.org/abs/2310.06770) |
| Dataset revision | `c104f840cc67f8c6eec6f759ebc8b2693d585d4a` |

## What the agent sees

The agent receives a task of the form: "Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Grading runs inside the per-task image after the agent exits.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/epoch-research/swe-bench.eval.x86_64.${EVAL_TASK_ID}:latest`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run swe-bench`
- `README.md` — this file
