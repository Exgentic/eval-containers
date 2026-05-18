# appworld

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/appworld-0-claude-code.trajectory.jsonl`](../../tests/fixtures/appworld-0-claude-code.trajectory.jsonl)


AppWorld - 9 simulated apps with 457 APIs

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 732 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/stonybrooknlp/appworld](https://github.com/stonybrooknlp/appworld) |
| Paper | [paper](https://arxiv.org/abs/2407.18901) |
| Dataset revision | `refs/convert/parquet` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run appworld`
- `README.md` — this file
