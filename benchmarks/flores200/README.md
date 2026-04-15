# flores200

FLORES-200 - multilingual machine translation from English to 10 target languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 10120 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/facebookresearch/flores](https://github.com/facebookresearch/flores) |
| Paper | [paper](https://arxiv.org/abs/2207.04672) |
| Dataset revision | `flores200_dataset-2022-07-14` |

## What the agent sees

The agent receives a task of the form: "Translate the following English sentence into ${LANG_NAME}. Print only the translated sentence with no explanation, quotes, or additional text." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run flores200`
- `README.md` — this file
