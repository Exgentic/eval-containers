# arc-agi

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/arc-agi-0-claude-code.traces.jsonl`](../../tests/fixtures/arc-agi-0-claude-code.traces.jsonl)


ARC-AGI-2 - Abstract grid reasoning tasks

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 167 (one per test input of the 120 puzzles) |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/arcprize/ARC-AGI-2](https://github.com/arcprize/ARC-AGI-2) |
| Paper | [paper](https://arxiv.org/abs/1911.01547) |
| Dataset revision | `f3283f727488ad98fe575ea6a5ac981e4a188e49` |

## What the agent sees

The agent receives a task of the form: "You are given a series of training examples showing input and output grids. Each grid is a 2D array of integers (0-9). Study the pattern in the training examples, then predict the output grid for the test input." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## Tasks

The evaluation set has 120 puzzles; 45 of them have two or three test inputs,
167 test inputs in all. As in the official scorer
([arcprize/arc-agi-benchmarking](https://github.com/arcprize/arc-agi-benchmarking)),
which grades each test output on its own, every test input is its own task: the
prompt shows the puzzle's training examples and that one test input.

- Tasks `0`–`119` are the first test input of each puzzle, in the sorted order
  of the upstream file names (the same numbering the 120-task version used).
- Tasks `120`–`166` are the remaining test inputs of the multi-test puzzles, in
  the same puzzle order, then by test index.
- `id.txt` is the upstream puzzle id (`142ca369`) for a single-test puzzle, and
  `<puzzle>_<k>` (`142ca369_0`, `142ca369_1`, k from 0) for a multi-test one.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

Differences from the official score: it takes 2 attempts per test input (pass@2)
while this is pass@1, and it scores a puzzle as the fraction of its test inputs
solved, averaged over the 120 puzzles, while the mean reward here is over the 167
test inputs. The official number can be computed from the per-task rewards by
grouping on the puzzle part of `id.txt`.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run arc-agi`
- `README.md` — this file
