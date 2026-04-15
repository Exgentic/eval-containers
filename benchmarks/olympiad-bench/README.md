# olympiad-bench

OlympiadBench - olympiad-level math and physics problems (text-only English split)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 910 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/Hothan/OlympiadBench](https://huggingface.co/datasets/Hothan/OlympiadBench) |
| Paper | [paper](https://arxiv.org/abs/2402.14008) |
| Dataset revision | `91184b52131e7fc9455fef848035173aea8cc01a` |

## What the agent sees

The agent receives a task of the form: "Solve this olympiad problem. Print only the final answer in its simplest form (a number, expression, or semicolon-separated list for multi-part answers). Do not include any explanation or units." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run olympiad-bench`
- `README.md` — this file
