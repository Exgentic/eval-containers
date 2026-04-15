# mathvista

MathVista - visual mathematical reasoning benchmark

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://mathvista.github.io](https://mathvista.github.io) |
| Paper | [paper](https://arxiv.org/abs/2310.02255) |
| Dataset revision | `2b6ad69445fbb5695c9b165475e8decdbeb97747` |

## What the agent sees

The agent receives a task of the form: "Look at the provided image and answer the following question. Print only the final answer, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run mathvista`
- `README.md` — this file
