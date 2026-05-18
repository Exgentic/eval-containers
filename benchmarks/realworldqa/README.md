# realworldqa

RealWorldQA - real-world spatial understanding benchmark from xAI

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 765 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/xai-org/RealworldQA](https://huggingface.co/datasets/xai-org/RealworldQA) |
| Paper | [paper](https://x.ai/blog/grok-1.5v) |
| Dataset revision | `17e7f75e092e47169732462ea3cdfebe911105dd` |

## What the agent sees

The agent receives a task of the form: "Look at the provided image and answer the following question about the real-world scene. Print only the final answer (a letter for multiple choice or a short phrase), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run realworldqa`
- `README.md` — this file
