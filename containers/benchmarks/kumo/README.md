# kumo

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/kumo-0-codex.traces.jsonl`](../../tests/fixtures/kumo-0-codex.traces.jsonl)


Kumo - diagnostic reasoning

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 250 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/Haowei-PKU/Kumo](https://github.com/Haowei-PKU/Kumo) |
| Paper | [paper](https://arxiv.org/abs/2505.08857) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Based on the observation, select the correct action. Print ONLY the action, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run kumo`
- `README.md` — this file
