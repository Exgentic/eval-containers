# coderefine

CodeRefine (CodeXGLUE) - Java bug-fix code refinement

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 6545 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/microsoft/CodeXGLUE/tree/main/Code-Code/code-refinement](https://github.com/microsoft/CodeXGLUE/tree/main/Code-Code/code-refinement) |
| Paper | [paper](https://arxiv.org/abs/2102.04664) |
| Dataset revision | `07ab797a018d0d5c448b56eb26b5e11aa5ad7659` |

## What the agent sees

The agent receives a task of the form: "You are given a buggy Java function. Print the refined (bug-free)" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run coderefine`
- `README.md` — this file
