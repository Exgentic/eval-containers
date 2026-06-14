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

The agent receives a task of the form: "Write a complete, self-contained Python solution for the following task. Print ONLY the Python code (including all needed imports and the full function definition), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/instruct_prompt.txt` and passed in via the `TASK` environment variable.

## How it's graded

`/grade.sh` (inline in the Dockerfile) assembles the agent's stdout + the task's
upstream test and grades them with the shared `/eval-grade` harness
(`benchmarks/RULES.md` 22). The reward is decided by the harness, not by the
candidate process's exit code: the reward file is seeded to `0.0` and `/eval-grade`
overwrites it with `1.0` only on a genuine unittest pass (read from `TestResult`
attributes), so a completion cannot self-pass by exiting cleanly or patching the
test runner (rule 5). The test suite is `0600` root-only and the agent runs
unprivileged, so the agent cannot read it.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run bigcodebench`
- `README.md` — this file
