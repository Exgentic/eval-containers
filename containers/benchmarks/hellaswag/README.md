# hellaswag

HellaSwag - commonsense sentence completion: pick the most plausible ending

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 10042 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/Rowan/hellaswag](https://huggingface.co/datasets/Rowan/hellaswag) |
| Paper | [paper](https://arxiv.org/abs/1905.07830) |
| Dataset revision | `218ec52e09a7e7462a5400043bb9a69a41d06b76` |

## What the agent sees

The agent receives a task of the form: "Pick the most plausible continuation of the context. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run hellaswag`
- `README.md` — this file
