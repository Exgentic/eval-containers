# scibench

SciBench - college-level scientific problems in math, physics, and chemistry

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 692 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/mandyyyyii/scibench](https://github.com/mandyyyyii/scibench) |
| Paper | [paper](https://arxiv.org/abs/2307.10635) |
| Dataset revision | `93931252bc1b71d495e67390235940643d926958` |

## What the agent sees

The agent receives a task of the form: "Solve this college-level science problem. Print only the final numeric answer as a single number (no units, no explanation). Expected unit: ${UNIT}." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run scibench`
- `README.md` — this file
