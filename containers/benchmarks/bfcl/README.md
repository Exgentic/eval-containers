# bfcl

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/bfcl-0-codex.traces.jsonl`](../../tests/fixtures/bfcl-0-codex.traces.jsonl)


Berkeley Function Calling Leaderboard

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2000 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/ShishirPatil/gorilla](https://github.com/ShishirPatil/gorilla) |
| Paper | [paper](https://arxiv.org/abs/2406.06840) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Call the correct function(s) based on the conversation. Print ONLY the function call(s) as JSON, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run bfcl`
- `README.md` — this file
