# hle

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/hle-0-claude-code.traces.jsonl`](../../tests/fixtures/hle-0-claude-code.traces.jsonl)


HLE - Humanity's Last Exam, hardest standardized AI evaluation

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2500 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/centerforaisafety/hle](https://github.com/centerforaisafety/hle) |
| Paper | [paper](https://arxiv.org/abs/2501.14249) |
| Dataset revision | `5a81a4c7271a2a2a312b9a690f0c2fde837e4c29` |

## What the agent sees

The agent receives a task of the form: "$PROMPT" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run hle`
- `README.md` — this file
