# aider-polyglot

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/aider-polyglot-0-aider.trajectory.jsonl`](../../tests/fixtures/aider-polyglot-0-aider.trajectory.jsonl)


Aider Polyglot - multi-language code editing (Exercism)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 225 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/Aider-AI/polyglot-benchmark](https://github.com/Aider-AI/polyglot-benchmark) |
| Paper | — |
| Dataset revision | `7e0611e77b54e2dea774cdc0aa00cf9f7ed6144f` |

## What the agent sees

The agent receives a task of the form: "Edit the following $LANGUAGE code file(s) to solve the exercise described below. Modify ONLY the listed file(s) in the current directory. Do not create new files." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run aider-polyglot`
- `README.md` — this file
