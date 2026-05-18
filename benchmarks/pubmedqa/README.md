# pubmedqa

PubMedQA - biomedical research question answering from abstracts

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/pubmedqa/pubmedqa](https://github.com/pubmedqa/pubmedqa) |
| Paper | [paper](https://arxiv.org/abs/1909.06146) |
| Dataset revision | `9001f2853fb87cab8d220904e0de81ac6973b318` |

## What the agent sees

The agent receives a task of the form: "Read the biomedical research abstract and answer the question. Print only one of: yes, no, or maybe. Nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run pubmedqa`
- `README.md` — this file
