# mmlu

MMLU - massive multitask multiple choice across 57 academic subjects

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 14042 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/cais/mmlu](https://huggingface.co/datasets/cais/mmlu) |
| Paper | [paper](https://arxiv.org/abs/2009.03300) |
| Dataset revision | `c30699e8356da336a370243923dbaf21066bb9fe` |

## What the agent sees

The agent receives a task of the form: "Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run mmlu`
- `README.md` — this file
