# chartqa

ChartQA - question answering about charts with visual and logical reasoning

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2500 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/HuggingFaceM4/ChartQA](https://huggingface.co/datasets/HuggingFaceM4/ChartQA) |
| Paper | [paper](https://arxiv.org/abs/2203.10244) |
| Dataset revision | `b605b6e08b57faf4359aeb2fe6a3ca595f99b6c5` |

## What the agent sees

The agent receives a task of the form: "Look at the provided chart image and answer the following question. Print only the final answer (a number or short phrase), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run chartqa`
- `README.md` — this file
