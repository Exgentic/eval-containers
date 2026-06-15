# humaneval

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/humaneval-0-claude-code.traces.jsonl`](../../tests/fixtures/humaneval-0-claude-code.traces.jsonl)


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

The agent receives a task of the form: "Complete the following Python function. Print ONLY the function body (the code that goes after the function signature), nothing else. Do not repeat the function signature or docstring." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

`/grade.sh` (inline in the Dockerfile) assembles the agent's stdout + the task's
upstream test and grades them with the shared `/eval-grade` harness
(`benchmarks/RULES.md` 22). The reward is decided by the harness, not by the
candidate process's exit code: the reward file is seeded to `0.0` and `/eval-grade`
overwrites it with `1.0` only if the assembled program runs to completion with no
exception, so a completion cannot self-pass by exiting cleanly or crashing
(rule 5). The test is `0600` root-only and the agent runs unprivileged, so the
agent cannot read it.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run humaneval`
- `README.md` — this file
