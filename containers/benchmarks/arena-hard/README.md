# arena-hard

Arena-Hard - pairwise chat eval vs gpt-4-0314 baseline with LLM judge

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 500 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/lmarena/arena-hard-auto](https://github.com/lmarena/arena-hard-auto) |
| Paper | [paper](https://arxiv.org/abs/2406.11939) |
| Dataset revision | `196f6b826783b3da7310e361a805fa36f0be83f3` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run arena-hard`
- `README.md` — this file
