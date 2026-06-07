# core-bench

CORE-Bench - reproducing scientific research code results

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 45 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/siegelz/core-bench](https://github.com/siegelz/core-bench) |
| Paper | [paper](https://arxiv.org/abs/2409.11363) |
| Dataset revision | `e32a2980e72fe6eb04ee04eb749458f570625663` |

## What the agent sees

The agent receives a task of the form: "You are reproducing results from a published scientific code capsule." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run core-bench`
- `README.md` — this file
