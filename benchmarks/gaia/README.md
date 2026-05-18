# gaia

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/gaia-0-goose.trajectory.jsonl`](../../tests/fixtures/gaia-0-goose.trajectory.jsonl)


GAIA - General AI Assistants

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 165 |
| Environment | shared-env |
| Internet required | true |
| Released | yes |
| Upstream | [https://github.com/gaia-benchmark/GAIA](https://github.com/gaia-benchmark/GAIA) |
| Paper | [paper](https://arxiv.org/abs/2311.12983) |
| Dataset revision | `682dd723ee1e1697e00360edccf2366dc8418dd9` |

## What the agent sees

The agent receives a task of the form: "Answer this question. Print only the answer, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run gaia`
- `README.md` — this file
