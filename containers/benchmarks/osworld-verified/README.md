# osworld-verified

OSWorld-Verified — the repaired OSWorld suite, on upstream's own desktop.

Same 369 ids and evaluators as the original OSWorld; 117 task definitions carry
repairs, and it is what the public leaderboard reports against, so a score
against the original is not comparable to one here.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 335 |
| Environment | shared-env |
| Internet required | true |
| Released | no |
| Upstream | [https://github.com/xlang-ai/OSWorld](https://github.com/xlang-ai/OSWorld) |
| Paper | [paper](https://arxiv.org/abs/2404.07972) |
| Dataset revision | `b138d348256078fa634fc3b73567a7337c793e6b` |

## The desktop

The `desktop` sidecar is the machine upstream scores against: the root
filesystem of the VM disk OSWorld publishes, run as a container instead of
booted (`core/osworld-desktop`). Its Ubuntu 22.04, GNOME session, apps and
their settings are upstream's; the image adds only a virtual display and the
script that starts the session. It is x86_64 only, as the disk is.

## What the agent sees

That desktop, through one HTTP face the sidecar serves on port 5000:
`GET /screenshot` returns a 1920x1080 PNG, `POST /act` runs pyautogui calls.
Nothing else — upstream's control server, which can run commands in the
session, listens on a unix socket that only setup and grading can open, and
`/act` parses the action and rejects anything that is not a pyautogui call on
literal arguments. The problem text is read from
`/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment
variable, with upstream's own note that the computer's password is `password`.

## How it's graded

`/grade.sh` runs the task's own postconfig and then upstream's
execution-based evaluator against the desktop, through `osworld_env.py`. The
reward is that evaluator's score. A grader crash, a missing task file or a
non-numeric score all become 0 — never an abort, never an unearned score.

Upstream's own two lists decide what is graded, which is why the count is 335
and not 369. `test_infeasible.json` names 26 tasks whose correct answer is to
report the task impossible; upstream rewards that only when the agent's last
action was `FAIL`, and grading here runs in a fresh process that never sees it.
`test_nogdrive.json` is the suite without the 8 tasks that need a Google
account's OAuth client, which cannot ship in an image, and upstream accepts
results on that list. Scoring either group 0 would understate every agent.

## Files

- `Dockerfile` — builds the benchmark image
- `osworld_env.py` — points upstream's harness at the sidecar instead of a VM
- `compose.yaml` — compose file for `eval-containers run osworld-verified`
- `benchmarks/_chart/presets/osworld-verified.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=osworld-verified`
- `README.md` — this file
