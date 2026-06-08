# gdpval

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/gdpval-0-claude-code.trajectory.jsonl`](../../tests/fixtures/gdpval-0-claude-code.trajectory.jsonl)


GDPval - professional knowledge work

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 220 |
| Environment | shared-env |
| Internet required | true |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/openai/gdpval](https://huggingface.co/datasets/openai/gdpval) |
| Paper | — |
| Dataset revision | `auto-parquet-ref` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run gdpval`
- `README.md` — this file
