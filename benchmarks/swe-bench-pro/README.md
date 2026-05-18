# swe-bench-pro

SWE-bench Pro - harder, professional-grade software engineering tasks

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 731 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/SWE-bench/SWE-bench](https://github.com/SWE-bench/SWE-bench) |
| Paper | [paper](https://arxiv.org/abs/2509.16941) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Grading runs inside the per-task image after the agent exits.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/swe-bench/swe-bench-pro.eval.x86_64.${EVAL_TASK_ID}:latest`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run swe-bench-pro`
- `README.md` — this file
