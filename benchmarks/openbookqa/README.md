# openbookqa

OpenBookQA - elementary science multiple choice

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://allenai.org/data/open-book-qa](https://allenai.org/data/open-book-qa) |
| Paper | [paper](https://arxiv.org/abs/1809.02789) |
| Dataset revision | `388097ea7776314e93a529163e0fea805b8a6454` |

## What the agent sees

The agent receives a task of the form: "Answer this multiple choice science question. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run openbookqa`
- `README.md` — this file
