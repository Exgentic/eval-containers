# mbppplus

MBPP+ - EvalPlus augmented MBPP with 35x more tests

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 378 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/evalplus/evalplus](https://github.com/evalplus/evalplus) |
| Paper | [paper](https://arxiv.org/abs/2305.01210) |
| Dataset revision | `b2d74c91837c3f2a20c1299ae98133cbe7cfa077` |

## What the agent sees

The agent receives a task of the form: "Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mbppplus`
- `README.md` — this file
