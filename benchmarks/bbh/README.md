# bbh

BIG-Bench Hard - 27 challenging BIG-Bench subtasks for chain-of-thought reasoning

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 6511 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/suzgunmirac/BIG-Bench-Hard](https://github.com/suzgunmirac/BIG-Bench-Hard) |
| Paper | [paper](https://arxiv.org/abs/2210.09261) |
| Dataset revision | `9ee07bd481feebf959a6b59d61ea57bdcf30964d` |

## What the agent sees

The agent receives a task of the form: "Solve the following BIG-Bench Hard problem. Print ONLY the final answer exactly as it should appear (no explanation, no extra text, no quotes)." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run bbh`
- `README.md` — this file
