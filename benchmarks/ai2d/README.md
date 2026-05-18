# ai2d

AI2D - multiple choice questions about science diagrams

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 3088 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/lmms-lab/ai2d](https://huggingface.co/datasets/lmms-lab/ai2d) |
| Paper | [paper](https://arxiv.org/abs/1603.07396) |
| Dataset revision | `c83a9b9692933aff8349157c88a413df9d02c4e5` |

## What the agent sees

The agent receives a task of the form: "Look at the provided diagram and answer the multiple choice question. Print only the answer letter (A, B, C, D, ...), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run ai2d`
- `README.md` — this file
