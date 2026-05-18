# mrcr

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/mrcr-0-claude-code.trajectory.jsonl`](../../tests/fixtures/mrcr-0-claude-code.trajectory.jsonl)


MRCR v2 - Multi-Round Coreference Resolution (long-context retrieval)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2400 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/openai/mrcr](https://huggingface.co/datasets/openai/mrcr) |
| Paper | [paper](https://arxiv.org/abs/2505.14993) |
| Dataset revision | `f4c69fae7cf81f7ca26b9fee34b392a50f6b8a1d` |

## What the agent sees

The agent receives a task of the form: "Your task is described in the file /app/task.txt — read it and follow the instructions inside."" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mrcr`
- `README.md` — this file
