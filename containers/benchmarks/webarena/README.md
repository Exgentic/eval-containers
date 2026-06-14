# webarena

WebArena Verified - web browsing with sidecars

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 812 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/web-arena-x/webarena](https://github.com/web-arena-x/webarena) |
| Paper | [paper](https://arxiv.org/abs/2307.13854) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "You are a web browsing agent. Complete this task by interacting with websites." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run webarena`
- `benchmarks/_chart/presets/webarena.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=webarena`
- `README.md` — this file
