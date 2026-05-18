# xnli

XNLI - cross-lingual natural language inference across 15 languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 75150 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/facebook/xnli](https://huggingface.co/datasets/facebook/xnli) |
| Paper | [paper](https://arxiv.org/abs/1809.05053) |
| Dataset revision | `b8dd5d7af51114dbda02c0e3f6133f332186418e` |

## What the agent sees

The agent receives a task of the form: "Given the premise and hypothesis, determine the relationship. Print only one word: entailment, neutral, or contradiction. Do not include any explanation." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run xnli`
- `README.md` — this file
