# agentharm

AgentHarm - harmfulness evaluation for LLM agents

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 176 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/ai-safety-institute/AgentHarm](https://huggingface.co/datasets/ai-safety-institute/AgentHarm) |
| Paper | [paper](https://arxiv.org/abs/2410.09024) |
| Dataset revision | `e23b3fe60a0da9037314b88e5ee3a0c054970dad` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run agentharm`
- `README.md` — this file
