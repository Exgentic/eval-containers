# drop

DROP - reading comprehension requiring discrete reasoning (counting, arithmetic, sorting) over paragraphs

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 9535 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/ucinlp/drop](https://huggingface.co/datasets/ucinlp/drop) |
| Paper | [paper](https://arxiv.org/abs/1903.00161) |
| Dataset revision | `95cda593fae71b60b5b19f82de3fcf3298c1239c` |

## What the agent sees

The agent receives a task of the form: "Read the passage and answer the question. Print only the final answer with no explanation. For numeric answers, print only the number. For span answers, print the exact text from the passage." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run drop`
- `README.md` — this file
