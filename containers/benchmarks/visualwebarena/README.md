# visualwebarena

VisualWebArena - multimodal web agent tasks with sidecars

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 910 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/web-arena-x/visualwebarena](https://github.com/web-arena-x/visualwebarena) |
| Paper | [paper](https://arxiv.org/abs/2401.13649) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "You are a multimodal web browsing agent. Complete this task by interacting" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run visualwebarena`
- `benchmarks/_chart/presets/visualwebarena.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=visualwebarena`
- `README.md` — this file
