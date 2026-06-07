# mle-bench

MLE-bench - Kaggle-style ML engineering tasks (OpenAI)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 75 |
| Environment | per-task |
| Internet required | true |
| Released | no |
| Upstream | [https://github.com/openai/mle-bench](https://github.com/openai/mle-bench) |
| Paper | [paper](https://arxiv.org/abs/2410.07095) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "You are an ML engineer. The competition data is mounted at /home/data (read-only). Train a model and write your predictions to /home/submission/submission.csv. You may install packages and use the network." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Grading runs inside the per-task image after the agent exits.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/openai/mle-bench.${EVAL_TASK_ID}:latest`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mle-bench`
- `README.md` — this file
