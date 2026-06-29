# swe-lancer

SWE-Lancer — real-world freelance software-engineering tasks (OpenAI). Each task
is a real Expensify (`Expensify/App`) bug; the agent fixes it and is graded by
the task's own Playwright end-to-end test.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 198 (OSS IC-SWE subset) |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [openai/preparedness](https://github.com/openai/preparedness) (`project/swelancer`) |
| Paper | [paper](https://arxiv.org/abs/2502.12115) |
| Per-task base | `docker.io/swelancer/swelancer_x86_<task>:releasev1` (Diamond split) |

The legacy repo `openai/SWELancer-Benchmark` now redirects to `openai/preparedness`.
The open-source `project/swelancer` ships 198 IC-SWE issues (each with a checked-in
`test.py` + `bug_reintroduce.patch`); the paper's full 1,488-task set and the
SWE-Manager variants are not part of this OSS subset.

## What the agent sees

The agent works in the Expensify checkout at `/app/expensify`, set up at the
task's commit with the bug re-introduced. It receives a `TASK` of the form: "You
are fixing a freelance software-engineering task in the Expensify codebase at
/app/expensify. Do NOT modify test files. …" followed by the issue description
(from the bundled `issue_data.json`, baked to `/tasks/0/problem.txt`). The real
task id (`ISSUE_ID`) and the upstream tests are **not** visible to the agent —
`/app/tests` is root-only and the agent is launched with `env -i` (rule 7).

## How it's graded

`grade.sh` brings up the upstream service stack (`run.sh` — Xvfb/VNC, pusher-fake,
nginx, mitmproxy certs), then runs the task's own suite via
`run_tests.yml` (`pytest issues/$ISSUE_ID/test.py`, after starting the npm dev
server + mitm replay) and records its exit code. **reward = 1 iff pytest passes**,
written to `/logs/verifier/reward.txt`.

The gold solution (`solution.sh`, mounted only at oracle time — never baked) is
upstream's documented reference: reverse the bug patch
(`patch -p1 -R < bug_reintroduce.patch`), read from the image's root-only
`/app/tests/issues/<id>/` so it matches the exact version the base was built from.

## Per-task build (rule 24g)

`build.sh <image> <task-id>` pulls OpenAI's prebuilt per-task image
(`swelancer/swelancer_x86_<task>:releasev1` — Expensify + bug + build already
baked) and overlays only the eval pipeline. No from-source build.

## Files

- `Dockerfile` — eval-pipeline overlay on the prebuilt per-task base
- `build.sh` — pull the per-task image + overlay
- `solution.sh` — gold (reverse the bug patch); mounted at oracle time only
- `grade.sh` — service stack + `run_tests.yml` → reward
- `entrypoint.sh` — hands the checkout to the agent + seeds `TASK`
- `compose.yaml` — compose file for `eval-containers run swe-lancer`
- `README.md` — this file
