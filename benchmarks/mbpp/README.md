# mbpp

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/mbpp-0-claude-code.trajectory.jsonl`](../../tests/fixtures/mbpp-0-claude-code.trajectory.jsonl)


MBPP - mostly basic Python problems

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/google-research/google-research/tree/master/mbpp](https://github.com/google-research/google-research/tree/master/mbpp) |
| Paper | [paper](https://arxiv.org/abs/2108.07732) |
| Dataset revision | `4bb6404fdc6cacfda99d4ac4205087b89d32030c` |

## What the agent sees

The agent receives a task of the form: "Write a Python function to solve the following problem. Print ONLY the complete Python code (function definition and any needed imports), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mbpp`
- `README.md` — this file
