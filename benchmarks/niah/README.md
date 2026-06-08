# niah

Needle in a Haystack - long-context retrieval across context lengths and depths

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 63 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/gkamradt/LLMTest_NeedleInAHaystack](https://github.com/gkamradt/LLMTest_NeedleInAHaystack) |
| Paper | — |
| Dataset revision | `7b90d285651b68d39a94f3d3bd3672f84192c989` |

## What the agent sees

The agent receives a task of the form: "Read the long context in /app/context.txt carefully. Then answer the question in /app/question.txt based solely on information found in the context. Print only the answer, nothing else."" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run niah`
- `README.md` — this file
