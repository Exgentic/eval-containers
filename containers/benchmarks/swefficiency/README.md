# swefficiency

SWE-fficiency - make a workload on a real repository faster without breaking its tests

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 100 (SWE-fficiency Lite) |
| Environment | per-task |
| Internet required | false |
| Hardware | CPU, linux/amd64 (4 vCPUs, 16 GB, as upstream) |
| Released | no |
| Upstream | [https://github.com/swefficiency/swefficiency](https://github.com/swefficiency/swefficiency) |
| Paper | [ICML 2026](https://icml.cc/virtual/2026/poster/66745) |
| Dataset revision | `7fbfe6c2f1f409afb17acc4c3f737991ec6e1c27` |

`tasks.txt` lists the 100 instance ids of SWE-fficiency Lite
(`swefficiency/swefficiency_lite`, revision `7d7c3dc1`); each task's row comes
from the full dataset, as upstream's Lite report takes it. Each task is built
`FROM` upstream's own image for that instance
(`ghcr.io/swefficiency/swefficiency-images:<instance id>`): the repository at
`/testbed` and its conda env `testbed`.

## What the agent sees

Upstream's instruction prompt (`scripts/inference/templates/cursor_instruction_prompt.txt.j2`):
the workload script and its `workload()` function, the repository's test command
and its rebuild command, and the rules — speed up the workload by editing the
repository, not the workload, without breaking tests. The template defaults the
repository path to `/testbed/repo`; it is rendered with `/testbed/`, where the
repository is.

The agent owns `/testbed` and the `testbed` env, because the rebuild command
(`pip install -e .`) writes into the env. The entrypoint first saves both, root
only, for grading.

## How it's graded

`/grade.sh` takes the prediction as upstream's inference harness does
(`git add -N . && git diff --binary <base>`) and runs `/tests/grade.py`, which is
upstream's own harness (`swefficiency` at `12d32a2`, with its `uv.lock`): it calls
`run_instance` twice, as `swefficiency eval` does (`run_perf`, `run_correctness`,
process isolation, warm-up, 7,200 s per step), then upstream's `report.evaluate_instance`.

- **Expert:** upstream's report needs the expert's speedup from a gold run on the
  same machine, and the dataset's `speedup` column is empty, so the expert patch is
  timed here first; its tests are not run, since only the agent's correctness counts.
- **Agent:** the same timing, then upstream's introspection guard and the covering
  tests reset to the base commit.
- **Fresh container:** `run_instance` expects a new container of the task image for
  each run. Its Docker calls are served in this one, and each "new container" is
  `/testbed` and the `testbed` env restored from the entrypoint's copy, so nothing
  the agent left outside its diff reaches the grader.

Timing follows upstream: before and after the patch, a warm-up and a measured run
of the workload, each repeat in its own forked process, and the mean of the
repeats. The speedup ratio (SR) is the agent's speedup over the expert's; a patch
that breaks a covering test, fails the guard, or does not apply counts as no
speedup.

The reward is **1 − 0.001/SR**, with SR floored at upstream's 0.001. A run's mean
reward converts back exactly to upstream's headline number, the harmonic mean of
SR: **0.001 / (1 − mean reward)**. Matching the expert scores 0.999 and doing
nothing scores 1 − 0.001 × (expert speedup), so differences sit in the third
decimal. Upstream's per-instance verdict goes to `task/swefficiency.json`, its
logs to `task/logs/`, and the diff to `task/patch.diff`.

## Isolation

Upstream's pandas images keep an `upstream` remote whose refs reach about a
thousand commits after the task's base, which can include the expert fix. The
build removes every ref that is not an ancestor of the base and prunes the
objects, and checks that nothing newer remains.

## Offline

Upstream grades with the network on; here neither the agent nor the grader has
it, so `offline.py` fetches at build what a task would fetch at grading:

- **Build requirements.** Newer astropy releases rebuild (`pip install -e .`)
  with pip's build isolation, which downloads `[build-system] requires`. They are
  downloaded to `/opt/wheelhouse`, and `PIP_FIND_LINKS`/`PIP_NO_INDEX` point pip
  there. The other repositories build in place and need nothing.
- **Test spec.** Upstream's `make_test_spec` downloads some repositories'
  environment files from GitHub (xarray, matplotlib); the spec is made at build
  and loaded by the grader.
- **Workload data.** One Lite workload (`pydata__xarray-9001`) `wget`s a zip on
  every run and deletes it after. The zip is fetched once, and `/usr/local/bin/wget`
  serves it from `/opt/workload-data`; the workload itself is unchanged.

## Timing

Upstream pins each task to 2 whole cores (4 vCPUs, `cpu_assignment.py`). Many
expert speedups are only a few percent, so on shared or throttled CPUs the noise
is larger than the speedups being measured: run this benchmark on dedicated cores.

## Oracle

`solution.sh` applies the expert patch. Its SR is 1 up to timing noise, so the
reward is about 0.999 and the CI oracle gate, which requires exactly 1.0, does not
cover this benchmark; a no-op scores 1 − 0.001 × (expert speedup).

## Files

- `Dockerfile` — builds one task's image `FROM` upstream's
- `build-task.py` — writes the task row and upstream's prompt at build time
- `offline.py` — fetches build requirements and workload data at build time
- `entrypoint.sh` — saves `/testbed` and the env, then hands them to the agent
- `grade.sh`, `grade.py` — the verifier
- `solution.sh` — oracle gold
- `tasks.txt` — the 100 Lite instance ids
- `compose.yaml` — compose file for `eval-containers run swefficiency`
- `../_chart/presets/swefficiency.yaml` — the k8s surface's resources and grading grace
