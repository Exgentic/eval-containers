# math-500

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/math-500-0-aider.traces.jsonl`](../../tests/fixtures/math-500-0-aider.traces.jsonl)


MATH-500 - competition-level math problems

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/hendrycks/math](https://github.com/hendrycks/math) |
| Paper | [paper](https://arxiv.org/abs/2103.03874) |
| Dataset revision | `6e4ed1a2a79af7d8630a6b768ec859cb5af4d3be` |

## What the agent sees

The agent receives a task of the form: "Solve this math problem. Print only the final answer in its simplest form (e.g. a number, fraction, or expression). Do not include any explanation." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run math-500`
- `README.md` — this file
