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

## Per-task sites

Each task touches only a subset of the six websites (most just one). The benchmark
declares a sidecar catalog in `benchmarks/_chart/presets/webarena.yaml` and a
committed task→sites map at `benchmarks/_chart/task-profiles/webarena.json`
(regenerate with `gen-task-profiles.py`).

- **k8s / job:** the chart self-resolves — `helm template --set benchmark=webarena --set task=<id>` (or `eval-containers run webarena --task-id <id> --mode job`) brings up only that task's site(s). No CLI logic involved.
- **compose:** `docker compose` can't compute the subset from `EVAL_TASK_ID`, so `EVAL_TASK_ID=<id> docker compose up` brings up the full site set — and works without the CLI (rule 1). To run lean locally, name the task's service(s), e.g. `docker compose up runner map`.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run webarena` (full site set; see Per-task sites)
- `gen-task-profiles.py` — regenerate the task→sites map from the pinned dataset
- `benchmarks/_chart/presets/webarena.yaml` — the sidecar catalog + always-on proxy, overlaid on the shared chart via `--set benchmark=webarena`
- `benchmarks/_chart/task-profiles/webarena.json` — task→sites map; the chart self-resolves the per-task sidecars from it
- `README.md` — this file
