# medqa

MedQA - USMLE-style medical licensing exam questions, 4 options

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1273 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/jind11/MedQA](https://github.com/jind11/MedQA) |
| Paper | [paper](https://arxiv.org/abs/2009.13081) |
| Dataset revision | `0fb93dd23a7339b6dcd27e241cb9b5eca62d4d18` |

## What the agent sees

The agent receives a task of the form: "Answer this USMLE-style medical question. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run medqa`
- `README.md` — this file
