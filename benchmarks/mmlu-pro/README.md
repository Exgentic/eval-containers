# mmlu-pro

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/mmlu-pro-0-openhands.trajectory.jsonl`](../../tests/fixtures/mmlu-pro-0-openhands.trajectory.jsonl)


MMLU-Pro - graduate-level knowledge, multiple choice

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 12032 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/TIGER-AI-Lab/MMLU-Pro](https://github.com/TIGER-AI-Lab/MMLU-Pro) |
| Paper | [paper](https://arxiv.org/abs/2406.01574) |
| Dataset revision | `54611cde22c74cca43dd78732198de6abe971398` |

## What the agent sees

The agent receives a task of the form: "Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, D, E, F, G, H, I, or J), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run mmlu-pro`
- `README.md` — this file
