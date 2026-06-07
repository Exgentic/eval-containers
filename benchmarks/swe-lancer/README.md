# swe-lancer

SWE-Lancer - real-world freelance software engineering tasks (OpenAI)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1488 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/openai/SWELancer-Benchmark](https://github.com/openai/SWELancer-Benchmark) |
| Paper | [paper](https://arxiv.org/abs/2502.12115) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "You are fixing a freelance software engineering task in the Expensify codebase. Work in the checkout under /app/expensify (or /workspace). Do NOT modify test files. When finished, your change must pass the upstream end-to-end tests." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Grading runs inside the per-task image after the agent exits.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/openai/swelancer.${EVAL_TASK_ID}:latest`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run swe-lancer`
- `README.md` — this file
