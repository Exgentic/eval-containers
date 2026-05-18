# humanevalplus

HumanEval+ - EvalPlus augmented HumanEval with 80x more tests

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 164 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/evalplus/evalplus](https://github.com/evalplus/evalplus) |
| Paper | [paper](https://arxiv.org/abs/2305.01210) |
| Dataset revision | `d32357cf319e50e9c8d8dab5ea876c72b0fd321b` |

## What the agent sees

The agent receives a task of the form: "Complete the following Python function. Print ONLY the function body (the code that goes after the function signature), nothing else. Do not repeat the function signature or docstring." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run humanevalplus`
- `README.md` — this file
