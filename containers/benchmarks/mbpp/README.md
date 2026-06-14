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

`/grade.sh` (inline in the Dockerfile) assembles the agent's stdout + the task's
upstream test and grades them with the shared `/eval-grade` harness
(`benchmarks/RULES.md` 22). The reward is decided by the harness, not by the
candidate process's exit code: the reward file is seeded to `0.0` and `/eval-grade`
overwrites it with `1.0` only if the assembled program runs to completion with no
exception, so a completion cannot self-pass by exiting cleanly or crashing
(rule 5). The tests are `0600` root-only and the agent runs unprivileged, so the
agent cannot read them.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mbpp`
- `README.md` — this file
