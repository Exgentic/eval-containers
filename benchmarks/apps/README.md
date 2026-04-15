# apps

APPS - Python coding problems across 3 difficulty tiers

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 5000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/hendrycks/apps](https://github.com/hendrycks/apps) |
| Paper | [paper](https://arxiv.org/abs/2105.09938) |
| Dataset revision | `21e74ddf8de1a21436da12e3e653065c5213e9d1` |

## What the agent sees

The agent receives a task of the form: "Solve this Python coding problem. Print ONLY the complete Python source code, nothing else. If starter code is given, use it.$STARTER" The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run apps`
- `README.md` — this file
