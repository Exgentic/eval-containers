# osworld-verified

OSWorld-Verified — the repaired OSWorld suite, on a real desktop in a container.

Same 369 ids and evaluators as the original OSWorld; 117 task definitions carry
repairs, and it is what the public leaderboard reports against, so a score
against the original is not comparable to one here.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 343 |
| Environment | shared-env |
| Internet required | true |
| Released | no |
| Upstream | [https://github.com/xlang-ai/OSWorld](https://github.com/xlang-ai/OSWorld) |
| Paper | [paper](https://arxiv.org/abs/2404.07972) |
| Dataset revision | `b138d348256078fa634fc3b73567a7337c793e6b` |

## What the agent sees

A real Ubuntu desktop, through one HTTP face the `desktop` sidecar serves on
port 5000: `GET /screenshot` returns a 1920x1080 PNG, `POST /act` runs one
pyautogui call. Nothing else — the control server that can run commands in the
session listens on a root-only unix socket that setup and grading use, and
`/act` parses the action and rejects anything that is not a pyautogui call on
literal arguments. The problem text is read from
`/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment
variable.

## How it's graded

`/grade.sh` runs the task's own postconfig and then upstream's
execution-based evaluator against the desktop, through `osworld_env.py`. The
reward is that evaluator's score. A grader crash, a missing task file or a
non-numeric score all become 0 — never an abort, never an unearned score.

The suite's 26 `infeasible` tasks are left out, which is why the count is 343
and not 369. Their correct answer is to report the task impossible, and
upstream rewards that only when the agent's last action was `FAIL` — grading
here runs in a fresh process that never sees it, so scoring them would mark a
correct refusal wrong and understate every agent by 7%.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run osworld-verified`
- `benchmarks/_chart/presets/osworld-verified.yaml` — this benchmark's bespoke k8s topology (sidecars/Deployments/Services), overlaid on the shared chart when rendered with `--set benchmark=osworld-verified`
- `README.md` — this file
