# arc-agi

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/arc-agi-0-claude-code.traces.jsonl`](../../tests/fixtures/arc-agi-0-claude-code.traces.jsonl)


ARC-AGI-2 - Abstract grid reasoning tasks

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 120 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/arcprize/ARC-AGI-2](https://github.com/arcprize/ARC-AGI-2) |
| Paper | [paper](https://arxiv.org/abs/1911.01547) |
| Dataset revision | `f3283f727488ad98fe575ea6a5ac981e4a188e49` |

## What the agent sees

The agent receives a task of the form: "You are given a series of training examples showing input and output grids. Each grid is a 2D array of integers (0-9). Study the pattern in the training examples, then predict the output grid for the test input." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run arc-agi`
- `README.md` — this file
