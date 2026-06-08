# scicode

SciCode - scientific research coding across 16 subdomains

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 65 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/scicode-bench/SciCode](https://github.com/scicode-bench/SciCode) |
| Paper | [paper](https://arxiv.org/abs/2407.13168) |
| Dataset revision | `4510f6a6aa27c43fad7b43da2c59602a86e88480` |

## What the agent sees

The agent receives a task of the form: "Solve this scientific research coding problem. Implement ALL sub-step functions in a single Python file. Use the given function headers and return lines exactly. Print ONLY the complete Python code (including the required imports), nothing else.$BG" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run scicode`
- `README.md` — this file
