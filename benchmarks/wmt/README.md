# wmt

WMT24++ - multilingual machine translation from English to 10 target locales

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 9600 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/google/wmt24pp](https://huggingface.co/datasets/google/wmt24pp) |
| Paper | [paper](https://arxiv.org/abs/2502.12404) |
| Dataset revision | `e65f5856b1de3319c748c15e5aec0bc2336ec3b0` |

## What the agent sees

The agent receives a task of the form: "Translate the following English text into ${LANG_NAME}. Print only the translated text with no explanation, quotes, or additional commentary." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run wmt`
- `README.md` — this file
