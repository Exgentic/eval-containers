# workarena

WorkArena - enterprise web workflows on ServiceNow

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 682 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/ServiceNow/WorkArena](https://github.com/ServiceNow/WorkArena) |
| Paper | [paper](https://arxiv.org/abs/2403.07718) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run workarena`
- `README.md` — this file
