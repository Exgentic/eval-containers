# appworld

**Status:** Released ✓ — sample trajectory: [`tests/run/replay/fixtures/appworld-292-terminus-2.traces.jsonl`](../../tests/run/replay/fixtures/appworld-292-terminus-2.traces.jsonl)


AppWorld - 9 simulated apps with 457 APIs

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 732 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/stonybrooknlp/appworld](https://github.com/stonybrooknlp/appworld) |
| Paper | [paper](https://arxiv.org/abs/2407.18901) |
| Dataset revision | `refs/convert/parquet` |

## What the agent sees

The agent receives a task in `TASK` made up of two parts:

1. Generic boilerplate explaining that a code-execution bridge is running
   locally at `http://127.0.0.1:8123/execute` — `POST {"code": "..."}` runs
   Python against the simulated apps (the `apis` object is bound), returning
   `{"output": ...}` / `{"error": ...}`. Variables persist across calls; the
   agent calls `apis.supervisor.complete_task()` when done.
2. The task instruction itself, read from `/tasks/$EVAL_TASK_ID/problem.txt`.

The agent never gets direct SDK/filesystem access to AppWorld and never
learns its real task id (`EVAL_TASK_ID`/`TASK_ID` are excluded from its
environment, per rule 7) — `bridge.py`, a root-owned background HTTP service
started by `/entrypoint.sh`, holds the one live AppWorld session for the
real task id and proxies only sanitized code execution. This also keeps the
agent away from each task's `ground_truth/{answer,private_data,test_data}.json`
and `evaluation.py`, which sit right next to the `dbs/*.jsonl` files it
legitimately needs to interact with.

## How it's graded

`world.evaluate()` — AppWorld's own state-based, fully local/offline check
(database assertions against the task's `ground_truth/`). `AppWorld()`
re-initializes by deleting its own output directory on construction, so
grading reopening a second session would discard the agent's progress;
instead `/grade.sh` runs as root after the agent's process has exited and
sends `bridge.py` `SIGUSR1` (not an HTTP route — signals respect uid, so
this stays unreachable to the agent) to evaluate the same live session it's
been mutating. Reward is `1.0` if `evaluate().success`, else `0.0` — not
externally graded.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run appworld`
- `README.md` — this file
