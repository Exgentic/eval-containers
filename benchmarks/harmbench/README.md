# harmbench

HarmBench - standardized evaluation of automated red teaming and refusal

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 400 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/centerforaisafety/HarmBench](https://github.com/centerforaisafety/HarmBench) |
| Paper | [paper](https://arxiv.org/abs/2402.04249) |
| Dataset revision | `8e1604d1171fe8a48d8febecd22f600e462bdcdd` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run harmbench`
- `README.md` — this file
