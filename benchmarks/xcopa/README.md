# xcopa

XCOPA - cross-lingual causal commonsense reasoning across 11 languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 5500 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/cambridgeltl/xcopa](https://huggingface.co/datasets/cambridgeltl/xcopa) |
| Paper | [paper](https://arxiv.org/abs/2005.00333) |
| Dataset revision | `042f78955ba48e6404616762fa6e05e839c3907a` |

## What the agent sees

The agent receives a task of the form: "Choose the more plausible alternative. Print only the letter A or B, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run xcopa`
- `README.md` — this file
