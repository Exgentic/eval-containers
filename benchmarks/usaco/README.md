# usaco

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/usaco-0-codex.trajectory.jsonl`](../../tests/fixtures/usaco-0-codex.trajectory.jsonl)


USACO - competitive programming

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 307 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/dapumptu/usaco_benchmark](https://huggingface.co/datasets/dapumptu/usaco_benchmark) |
| Paper | — |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Solve this competitive programming problem. Print ONLY the complete source code (Python or C++), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run usaco`
- `README.md` — this file
