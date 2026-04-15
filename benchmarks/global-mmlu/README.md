# global-mmlu

Global-MMLU - multilingual knowledge and reasoning multiple choice across 42 languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 589764 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/CohereLabs/Global-MMLU](https://huggingface.co/datasets/CohereLabs/Global-MMLU) |
| Paper | [paper](https://arxiv.org/abs/2412.03304) |
| Dataset revision | `0e619dbeb34206cd48705a1a0ea7fb21cae09993` |

## What the agent sees

The agent receives a task of the form: "Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run global-mmlu`
- `README.md` — this file
