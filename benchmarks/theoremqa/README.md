# theoremqa

TheoremQA - STEM problems requiring the application of named theorems

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 800 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/TIGER-Lab/TheoremQA](https://huggingface.co/datasets/TIGER-Lab/TheoremQA) |
| Paper | [paper](https://arxiv.org/abs/2305.12524) |
| Dataset revision | `a340b1782960a712843aae3ed25f1e013cc008a5` |

## What the agent sees

The agent receives a task of the form: "Solve this theorem-based problem. Print only the final answer (a number, expression, or value). Do not include any explanation or units." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run theoremqa`
- `README.md` — this file
