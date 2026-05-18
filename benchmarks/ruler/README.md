# ruler

RULER - synthetic long-context benchmark (NIAH variants at multiple lengths)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 200 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/hsiehjackson/RULER](https://github.com/hsiehjackson/RULER) |
| Paper | [paper](https://arxiv.org/abs/2404.06654) |
| Dataset revision | `ab17b7853df4e0a30b78cd5d2b463ac7dff6ee13` |

## What the agent sees

The agent receives a task of the form: "Read the long context in /app/context.txt carefully. Then answer the question in /app/question.txt based solely on information found in the context. Print only the answer tokens/values, with no explanation."" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run ruler`
- `README.md` — this file
