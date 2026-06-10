# swe-bench

SWE-bench Verified - software engineering

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | per-task |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/princeton-nlp/SWE-bench](https://github.com/princeton-nlp/SWE-bench) |
| Paper | [paper](https://arxiv.org/abs/2310.06770) |
| Dataset revision | `c104f840cc67f8c6eec6f759ebc8b2693d585d4a` |

## What the agent sees

The agent receives a task of the form: "Fix this GitHub issue in the repository at /testbed. Edit the source code to resolve the bug. Do NOT modify any test files." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable. Because this benchmark uses a per-task environment, each task builds a separate image; the agent works inside the checked-out upstream base and must modify files in place.

## How it's graded

`/grade.sh` runs inside the per-task image after the agent exits and reuses the
**official SWE-bench harness** — no bespoke eval or grading logic. Two linear
steps:

1. **Run** (`bash`): execute `/tests/eval.sh` — swebench's own per-instance eval
   script, baked at build time from `test_spec.eval_script` (the exact script
   `run_evaluation` runs inside its container: activate conda, reinstall the
   patched package, reset the test files, apply the `test_patch`, run the test
   command). Output is captured to `/logs/verifier/test_output.log`.
2. **Grade** (`python`): `/tests/grade.py` scores that log with swebench's own
   `get_eval_report` (the same call `run_evaluation` makes) and prints the
   `reward` — `1` iff the instance resolves, else `0`.

The only difference from `run_evaluation` is *where* the eval script runs: in
this container (which already **is** the testbed, with the agent's edits applied
in place) instead of a fresh one `run_evaluation` would launch — there is no
Docker daemon inside the eval container, and this keeps grading self-contained
across the compose / container / job run modes. Python is used only where
swebench requires it (its log parser + report are a library, not a CLI); the
eval procedure itself stays bash.


## Per-task build

This benchmark uses `env=per-task`: the Dockerfile takes a `EVAL_TASK_ID` build-arg and pulls a per-task upstream base (`ghcr.io/epoch-research/swe-bench.eval.x86_64.${EVAL_TASK_ID}:latest`).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run swe-bench`
- `README.md` — this file
