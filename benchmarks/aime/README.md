# aime

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/aime-0-claude-code.trajectory.jsonl`](../../tests/fixtures/aime-0-claude-code.trajectory.jsonl)


American Invitational Mathematics Examination

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 90 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/AI-MO/aimo-validation-aime](https://huggingface.co/datasets/AI-MO/aimo-validation-aime) |
| Paper | — |
| Dataset revision | `13f9e12f613e720c2a2b2f345dd04b998a29494d` |

## What the agent sees

The agent receives a task of the form: "Solve this problem. Print only the answer as a single integer." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run aime`
- `README.md` — this file
