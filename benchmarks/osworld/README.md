# osworld

OSWorld - real computer environment for multimodal agents

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 369 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/xlang-ai/OSWorld](https://github.com/xlang-ai/OSWorld) |
| Paper | [paper](https://arxiv.org/abs/2404.07972) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "You are a computer use agent. Complete this task using the desktop." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run osworld`
- `README.md` — this file
