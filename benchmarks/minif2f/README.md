# minif2f

MiniF2F - formal-math competition problems (olympiad, AMC, AIME) with informal statements

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 244 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/yangky11/miniF2F](https://github.com/yangky11/miniF2F) |
| Paper | [paper](https://arxiv.org/abs/2109.00110) |
| Dataset revision | `c605019c4ebfda74d37bbb83e9c5774fc8b67c14` |

## What the agent sees

The agent receives a task of the form: "Solve this competition math problem. Print only the final answer in its simplest form (a number or expression). Do not include any explanation." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run minif2f`
- `README.md` — this file
