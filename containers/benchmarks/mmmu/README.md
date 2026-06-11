# mmmu

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/mmmu-0-claude-code.trajectory.jsonl`](../../tests/fixtures/mmmu-0-claude-code.trajectory.jsonl)


MMMU - massive multi-discipline multimodal understanding

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 900 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/MMMU-Benchmark/MMMU](https://github.com/MMMU-Benchmark/MMMU) |
| Paper | [paper](https://arxiv.org/abs/2311.16502) |
| Dataset revision | `21d1d90a93c7450d30bddb579d7b510c00b8a9ab` |

## What the agent sees

The agent receives a task of the form: "Look at the provided image(s) and answer this multiple choice question. Print only the answer letter (A, B, C, D, etc.), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mmmu`
- `README.md` — this file
