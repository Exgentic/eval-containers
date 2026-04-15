# mind2web

Mind2Web - generalist web agent instruction following

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1009 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/OSU-NLP-Group/Mind2Web](https://github.com/OSU-NLP-Group/Mind2Web) |
| Paper | [paper](https://arxiv.org/abs/2306.06070) |
| Dataset revision | `17ece8eb89862368edc0cc806acee6fca5163474` |

## What the agent sees

The agent receives a task of the form: "You are a generalist web agent. Produce a step-by-step action trace that" The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run mind2web`
- `README.md` — this file
