# livecodebench

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/livecodebench-0-codex.trajectory.jsonl`](../../tests/fixtures/livecodebench-0-codex.trajectory.jsonl)


LiveCodeBench - competitive programming from recent contests

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 880 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/LiveCodeBench/LiveCodeBench](https://github.com/LiveCodeBench/LiveCodeBench) |
| Paper | [paper](https://arxiv.org/abs/2403.07974) |
| Dataset revision | `0fe84c3912ea0c4d4a78037083943e8f0c4dd505` |

## What the agent sees

The agent receives a task of the form: "Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else.$STARTER" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run livecodebench`
- `README.md` — this file
