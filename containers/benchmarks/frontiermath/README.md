# frontiermath

FrontierMath - research-level math problems (Epoch AI)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 10 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://epochai.org/frontiermath](https://epochai.org/frontiermath) |
| Paper | [paper](https://arxiv.org/abs/2411.04872) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Solve this research-level mathematics problem. Print only the final answer on the last line." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run frontiermath`
- `README.md` — this file
