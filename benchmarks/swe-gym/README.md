# swe-gym

SWE-Gym - 2438 Python SWE tasks for agent training / eval

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2438 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/SWE-Gym/SWE-Gym](https://github.com/SWE-Gym/SWE-Gym) |
| Paper | [paper](https://arxiv.org/abs/2412.21139) |
| Dataset revision | `bb94ed9e39bbeb96a7fcbfb533b80f25a7fd59cb` |

## What the agent sees

The agent receives a task of the form: "Fix this GitHub issue in the Python repository $REPO at commit $COMMIT." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run swe-gym`
- `README.md` — this file
