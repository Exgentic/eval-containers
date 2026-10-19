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
Upstream's control server, which can run commands in the session, listens on a
unix socket that only setup and grading can open, and `/act` parses the action
and rejects anything that is not a pyautogui call on literal arguments.

One thing escapes that. 71 tasks start Chrome with a debugging port: the
task's setup opens its pages through it, and 22 graders read the browser
through it. The runner shares the desktop's network, so the agent can reach
that port too and script the browser instead of clicking. Upstream's agents can
do as much — their actions are Python run inside the VM.

The problem text is read from
`/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment
variable, with upstream's own note that the computer's password is `password`.

## How it's graded

`/grade.sh` runs the task's own postconfig and then upstream's
execution-based evaluator against the desktop, through the harness image's
`osworld_env.py`. The
reward is that evaluator's score. A grader crash, a missing task file or a
non-numeric score all become 0 — never an abort, never an unearned score.

The grader is checked by the oracle (`eval-containers oracle osworld-verified
--task-id <n>`): a gold solution must score 1 and a no-op less. Upstream
publishes no reference solutions, so a task's gold is the actions one passing
agent run sent to the desktop, replayed at that run's own pace on a fresh one —
`solutions/<upstream id>.jsonl`, written from a run's call record by
`solution-from-calls.py`. A task has one once an agent has solved it.

The harness, the task definitions and each task's files — what its setup opens
and what its grader compares against — are in the image, root-only, at pinned
revisions; upstream fetches the files from `xlangai/ubuntu_osworld_file_cache`
as it goes, and here those requests are answered from disk.

Upstream's own two lists decide what is graded, which is why the count is 335
and not 369. `test_infeasible.json` names 26 tasks whose correct answer is to
report the task impossible; upstream rewards that only when the agent's last
action was `FAIL`, and grading here runs in a fresh process that never sees it.
`test_nogdrive.json` is the suite without the 8 tasks that need a Google
account's OAuth client, which cannot ship in an image, and upstream accepts
results on that list. Scoring either group 0 would understate every agent.

## Before reading a score

`AUDIT.md` holds what has been checked and the evidence. Three things in it
change what a number here means:

- **Three tasks pass with no action** (39, 231, 270). They are upstream's
  setups and graders and are graded as upstream grades them.
- **49 tasks browse the live web, and upstream recommends a residential proxy
  for them.** Here the desktop goes out directly. Four are refused at their
  first page: two because of the cluster's address (6, 26), two because the
  desktop's Chrome is two years old (36, 40), as it is in upstream's VM. One
  task's start page no longer exists (15); the page it is graded on does.
- **Upstream publishes no reference solutions.** Ten tasks have been passed
  here; nothing shows the other 325 can be.

## Files

- `Dockerfile` — names the benchmark on top of `core/osworld-harness`, which holds the harness, the task files and the scripts that set a task up and grade it
- `solution.sh`, `solutions/`, `solution-from-calls.py` — the oracle's gold solutions; never in the image
- `compose.yaml` — compose file for `eval-containers run osworld-verified`
- `benchmarks/_chart/presets/osworld-verified.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=osworld-verified`
- `AUDIT.md` — what has been checked about this benchmark, and what has not
- `README.md` — this file
