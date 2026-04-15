# medmcqa

MedMCQA - multi-subject multi-choice medical entrance exam questions

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 4183 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/MedMCQA/MedMCQA](https://github.com/MedMCQA/MedMCQA) |
| Paper | [paper](https://arxiv.org/abs/2203.14371) |
| Dataset revision | `91c6572c454088bf71b679ad90aa8dffcd0d5868` |

## What the agent sees

The agent receives a task of the form: "Answer this medical multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run medmcqa`
- `README.md` — this file
