# docvqa

DocVQA - question answering on document images

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 5349 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://www.docvqa.org](https://www.docvqa.org) |
| Paper | [paper](https://arxiv.org/abs/2007.00398) |
| Dataset revision | `539088ef8a8ada01ac8e2e6d4e372586748a265e` |

## What the agent sees

The agent receives a task of the form: "Look at the provided document image and answer the following question. Print only the answer text (a short phrase or number copied from the document), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run docvqa`
- `README.md` — this file
