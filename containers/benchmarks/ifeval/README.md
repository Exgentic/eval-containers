# ifeval

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/ifeval-0-claude-code.traces.jsonl`](../../tests/fixtures/ifeval-0-claude-code.traces.jsonl)


IFEval - instruction following with verifiable constraints

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 541 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/google-research/google-research/tree/master/instruction_following_eval](https://github.com/google-research/google-research/tree/master/instruction_following_eval) |
| Paper | [paper](https://arxiv.org/abs/2311.07911) |
| Dataset revision | `966cd89545d6b6acfd7638bc708b98261ca58e84` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/prompt.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run ifeval`
- `README.md` — this file
