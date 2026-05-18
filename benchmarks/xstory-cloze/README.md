# xstory-cloze

XStoryCloze - cross-lingual commonsense story completion across 11 languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 16621 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/juletxara/xstory_cloze](https://huggingface.co/datasets/juletxara/xstory_cloze) |
| Paper | [paper](https://arxiv.org/abs/2112.10668) |
| Dataset revision | `c4c2d88a1ec8b37fe22166d2a610f272726724b6` |

## What the agent sees

The agent receives a task of the form: "Read the story and choose the correct ending. Print only the number 1 or 2, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run xstory-cloze`
- `README.md` — this file
