# bigcodebench

BigCodeBench - practical Python function generation with rich tool calls

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1140 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/bigcode-project/bigcodebench](https://github.com/bigcode-project/bigcodebench) |
| Paper | [paper](https://arxiv.org/abs/2406.15877) |
| Dataset revision | `b74c0d0bf70d2c0bc459be537895cca163007f1a` |

## What the agent sees

The agent receives a task of the form: "Write a complete, self-contained Python solution for the following task. Print ONLY the Python code (including all needed imports and the full function definition), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run bigcodebench`
- `README.md` — this file
