# gpqa-diamond

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/gpqa-diamond-0-codex.trajectory.jsonl`](../../tests/fixtures/gpqa-diamond-0-codex.trajectory.jsonl)


GPQA Diamond - graduate-level science

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 198 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/idavidrein/gpqa](https://github.com/idavidrein/gpqa) |
| Paper | [paper](https://arxiv.org/abs/2311.12022) |
| Dataset revision | `68be7564497676e07a77a042fdb587deb88c51c3` |

## What the agent sees

The agent receives a task of the form: "Answer this multiple choice question. Print only the letter (A, B, C, or D), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run gpqa-diamond`
- `README.md` — this file
