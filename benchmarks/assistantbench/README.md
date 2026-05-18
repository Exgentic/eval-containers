# assistantbench

AssistantBench - realistic long-horizon web research

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 33 |
| Environment | shared-env |
| Internet required | true |
| Released | no |
| Upstream | [https://github.com/oriyor/assistant-bench](https://github.com/oriyor/assistant-bench) |
| Paper | [paper](https://arxiv.org/abs/2407.15711) |
| Dataset revision | `482cbbc0400f6d048438c4021727f21a10cbff49` |

## What the agent sees

The agent receives a task of the form: "Browse the web to answer this question. Print only the answer, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run assistantbench`
- `README.md` — this file
