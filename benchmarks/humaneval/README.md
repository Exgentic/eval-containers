# humaneval

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/humaneval-0-claude-code.trajectory.jsonl`](../../tests/fixtures/humaneval-0-claude-code.trajectory.jsonl)


HumanEval - Python code generation

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 164 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/openai/human-eval](https://github.com/openai/human-eval) |
| Paper | [paper](https://arxiv.org/abs/2107.03374) |
| Dataset revision | `7dce6050a7d6d172f3cc5c32aa97f52fa1a2e544` |

## What the agent sees

The agent receives a task of the form: "Complete the following Python function. Print ONLY the function body (the code that goes after the function signature), nothing else. Do not repeat the function signature or docstring." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run humaneval`
- `README.md` — this file
